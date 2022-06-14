use crate::web2::tcp::{Listener, ToListener};
use crate::Server;

use std::fmt::{self, Debug, Display, Formatter};

use async_std::io;

use crate::web2::tcp::ListenInfo;

#[derive(Default)]
pub struct FailoverListener<State> {
    listeners: Vec<Option<Box<dyn Listener<State>>>>,
    index: Option<usize>,
}

impl<State> FailoverListener<State>
where
    State: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            listeners: vec![],
            index: None,
        }
    }

    pub fn add<L>(&mut self, listener: L) -> io::Result<()>
    where
        L: ToListener<State>,
    {
        self.listeners.push(Some(Box::new(listener.to_listener()?)));
        Ok(())
    }

    pub fn with_listener<L>(mut self, listener: L) -> Self
    where
        L: ToListener<State>,
    {
        self.add(listener).expect("Unable to add listener");
        self
    }
}

#[async_trait::async_trait]
impl<State> Listener<State> for FailoverListener<State>
where
    State: Clone + Send + Sync + 'static,
{
    async fn bind(&mut self, app: Server<State>) -> io::Result<()> {
        for (index, listener) in self.listeners.iter_mut().enumerate() {
            let listener = listener.as_deref_mut().expect("bind called twice");
            match listener.bind(app.clone()).await {
                Ok(_) => {
                    self.index = Some(index);
                    return Ok(());
                }
                Err(e) => {
                    crate::log::info!("unable to bind", {
                        listener: listener.to_string(),
                        error: e.to_string()
                    });
                }
            }
        }

        Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "unable to bind to any supplied listener spec",
        ))
    }

    async fn accept(&mut self) -> io::Result<()> {
        match self.index {
            Some(index) => {
                let mut listener = self.listeners[index].take().expect("accept called twice");
                listener.accept().await?;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "unable to listen to any supplied listener spec",
            )),
        }
    }

    fn info(&self) -> Vec<ListenInfo> {
        match self.index {
            Some(index) => match self.listeners.get(index) {
                Some(Some(listener)) => listener.info(),
                _ => vec![],
            },
            None => vec![],
        }
    }
}

impl<State> Debug for FailoverListener<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.listeners)
    }
}

impl<State> Display for FailoverListener<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = self
            .listeners
            .iter()
            .map(|l| match l {
                Some(l) => l.to_string(),
                None => String::new(),
            })
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(f, "{}", string)
    }
}