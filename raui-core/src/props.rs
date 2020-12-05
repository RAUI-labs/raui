use std::{any::Any, borrow::Cow};

pub enum PropsError {
    CouldNotReadData,
}

pub trait PropsData: Any {
    fn clone_props(&self) -> Box<dyn PropsData>;
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn PropsData> {
    fn clone(&self) -> Self {
        self.clone_props()
    }
}

#[derive(Clone)]
pub struct Props(Box<dyn PropsData>);

impl Default for Props {
    fn default() -> Self {
        Self(Box::new(()))
    }
}

impl std::fmt::Debug for Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Props {...}")
    }
}

impl Props {
    pub fn new<T>(data: T) -> Self
    where
        T: 'static + PropsData,
    {
        Self(Box::new(data))
    }

    pub fn read<T>(&self) -> Result<&T, PropsError>
    where
        T: 'static + PropsData,
    {
        if let Some(data) = self.0.as_any().downcast_ref::<T>() {
            Ok(data)
        } else {
            Err(PropsError::CouldNotReadData)
        }
    }

    pub fn read_cloned<T>(&self) -> Result<T, PropsError>
    where
        T: 'static + PropsData + Clone,
    {
        if let Some(data) = self.0.as_any().downcast_ref::<T>() {
            Ok(data.clone())
        } else {
            Err(PropsError::CouldNotReadData)
        }
    }

    pub fn read_cloned_or_default<T>(&self) -> T
    where
        T: 'static + PropsData + Clone + Default,
    {
        self.read_cloned().unwrap_or_default()
    }
}

#[macro_export]
macro_rules! implement_props_data {
    ($type_name:ty) => {
        impl $crate::props::PropsData for $type_name
        where
            Self: Clone,
        {
            fn clone_props(&self) -> Box<dyn $crate::props::PropsData> {
                Box::new(self.clone())
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

implement_props_data!(());
implement_props_data!(i8);
implement_props_data!(i16);
implement_props_data!(i32);
implement_props_data!(i64);
implement_props_data!(i128);
implement_props_data!(u8);
implement_props_data!(u16);
implement_props_data!(u32);
implement_props_data!(u64);
implement_props_data!(u128);
implement_props_data!(f32);
implement_props_data!(f64);
implement_props_data!(bool);
implement_props_data!(String);

impl<T> PropsData for Option<T>
where
    T: PropsData + Clone,
{
    fn clone_props(&self) -> Box<dyn PropsData> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T, E> PropsData for Result<T, E>
where
    T: PropsData + Clone,
    E: 'static + Clone,
{
    fn clone_props(&self) -> Box<dyn PropsData> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T> PropsData for Cow<'static, T>
where
    T: PropsData + Clone,
{
    fn clone_props(&self) -> Box<dyn PropsData> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
