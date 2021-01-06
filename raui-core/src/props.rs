use serde::{Deserialize, Serialize};
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

#[derive(Debug, Clone)]
pub enum PropsError {
    CouldNotReadData,
    HasNoDataOfType(String),
}

#[typetag::serde(tag = "type", content = "value")]
pub trait PropsData: std::fmt::Debug + Send + Sync {
    fn clone_props(&self) -> Box<dyn PropsData>;
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn PropsData> {
    fn clone(&self) -> Self {
        self.clone_props()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PropsDef(pub HashMap<String, Box<dyn PropsData>>);

#[derive(Debug, Default, Clone)]
pub struct Props(HashMap<TypeId, Box<dyn PropsData>>);

impl Props {
    pub(crate) fn from_raw(map: HashMap<TypeId, Box<dyn PropsData>>) -> Self {
        Self(map)
    }

    pub fn new<T>(data: T) -> Self
    where
        T: 'static + PropsData,
    {
        let mut result = HashMap::with_capacity(1);
        result.insert(TypeId::of::<T>(), Box::new(data) as Box<dyn PropsData>);
        Self(result)
    }

    pub fn has<T>(&self) -> bool
    where
        T: 'static + PropsData,
    {
        let e = TypeId::of::<T>();
        self.0.iter().any(|(t, _)| *t == e)
    }

    pub fn consume<T>(&mut self) -> Result<Box<dyn PropsData>, PropsError>
    where
        T: 'static + PropsData,
    {
        if let Some(v) = self.0.remove(&TypeId::of::<T>()) {
            Ok(v)
        } else {
            Err(PropsError::HasNoDataOfType(type_name::<T>().to_owned()))
        }
    }

    pub fn read<T>(&self) -> Result<&T, PropsError>
    where
        T: 'static + PropsData,
    {
        let e = TypeId::of::<T>();
        if let Some((_, v)) = self.0.iter().find(|(t, _)| **t == e) {
            if let Some(data) = v.as_any().downcast_ref::<T>() {
                Ok(data)
            } else {
                Err(PropsError::CouldNotReadData)
            }
        } else {
            Err(PropsError::HasNoDataOfType(type_name::<T>().to_owned()))
        }
    }

    pub fn map_or_default<T, R, F>(&self, mut f: F) -> R
    where
        T: 'static + PropsData,
        R: Default,
        F: FnMut(&T) -> R,
    {
        match self.read() {
            Ok(data) => f(data),
            Err(_) => R::default(),
        }
    }

    pub fn map_or_else<T, R, F, E>(&self, mut f: F, mut e: E) -> R
    where
        T: 'static + PropsData,
        F: FnMut(&T) -> R,
        E: FnMut() -> R,
    {
        match self.read() {
            Ok(data) => f(data),
            Err(_) => e(),
        }
    }

    pub fn read_cloned<T>(&self) -> Result<T, PropsError>
    where
        T: 'static + PropsData + Clone,
    {
        self.read::<T>().map(|v| v.clone())
    }

    pub fn read_cloned_or_default<T>(&self) -> T
    where
        T: 'static + PropsData + Clone + Default,
    {
        self.read_cloned().unwrap_or_default()
    }

    pub fn write<T>(&mut self, data: T)
    where
        T: 'static + PropsData,
    {
        self.0
            .insert(TypeId::of::<T>(), Box::new(data) as Box<dyn PropsData>);
    }

    pub fn with<T>(mut self, data: T) -> Self
    where
        T: 'static + PropsData,
    {
        self.write(data);
        self
    }

    pub fn without<T>(mut self) -> Self
    where
        T: 'static + PropsData,
    {
        self.0.remove(&TypeId::of::<T>());
        self
    }

    pub fn merge(self, other: Self) -> Self {
        let mut result = self.into_inner();
        result.extend(other.into_inner());
        Self(result)
    }

    pub(crate) fn into_inner(self) -> HashMap<TypeId, Box<dyn PropsData>> {
        self.0
    }
}

impl<T> From<T> for Props
where
    T: 'static + PropsData,
{
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl From<&Self> for Props {
    fn from(data: &Self) -> Self {
        data.clone()
    }
}

#[macro_export]
macro_rules! implement_props_data {
    ($type_name:ty, $tag_name:literal) => {
        #[$crate::typetag::serde(name = $tag_name)]
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

implement_props_data!((), "()");
implement_props_data!(i8, "i8");
implement_props_data!(i16, "i16");
implement_props_data!(i32, "i32");
implement_props_data!(i64, "i64");
implement_props_data!(i128, "i128");
implement_props_data!(u8, "u8");
implement_props_data!(u16, "u16");
implement_props_data!(u32, "u32");
implement_props_data!(u64, "u64");
implement_props_data!(u128, "u128");
implement_props_data!(f32, "f32");
implement_props_data!(f64, "f64");
implement_props_data!(bool, "bool");
implement_props_data!(String, "String");
