use crate::{Prefab, PrefabError, PrefabValue};
use serde::{Deserialize, Serialize};
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

type PropsSerializeFactory =
    Box<dyn Fn(&dyn PropsData) -> Result<PrefabValue, PrefabError> + Send + Sync>;
type PropsDeserializeFactory =
    Box<dyn Fn(PrefabValue, &mut Props) -> Result<(), PrefabError> + Send + Sync>;

#[derive(Default)]
pub struct PropsRegistry {
    type_mapping: HashMap<TypeId, String>,
    factories: HashMap<String, (PropsSerializeFactory, PropsDeserializeFactory)>,
}

impl PropsRegistry {
    pub fn register_factory<T>(&mut self, name: &str)
    where
        T: 'static + Prefab + PropsData,
    {
        let s: PropsSerializeFactory = Box::new(move |data| {
            if let Some(data) = data.as_any().downcast_ref::<T>() {
                data.to_prefab()
            } else {
                Err(PrefabError::CouldNotSerialize(
                    "Could not downcast to concrete type!".to_owned(),
                ))
            }
        });
        let d: PropsDeserializeFactory = Box::new(move |data, props| {
            props.write(T::from_prefab(data)?);
            Ok(())
        });
        self.factories.insert(name.to_owned(), (s, d));
        self.type_mapping.insert(TypeId::of::<T>(), name.to_owned());
    }

    pub fn unregister_factory(&mut self, name: &str) {
        self.factories.remove(name);
    }

    pub fn serialize(&self, props: &Props) -> Result<PrefabValue, PrefabError> {
        let mut group = PropsGroupPrefab::default();
        for (t, p) in &props.0 {
            if let Some(name) = self.type_mapping.get(t) {
                if let Some(factory) = self.factories.get(name) {
                    group.data.insert(name.to_owned(), (factory.0)(p.as_ref())?);
                }
            } else {
                return Err(PrefabError::CouldNotSerialize(
                    "No type mapping found!".to_owned(),
                ));
            }
        }
        group.to_prefab()
    }

    pub fn deserialize(&self, data: PrefabValue) -> Result<Props, PrefabError> {
        let data = if data.is_null() {
            PropsGroupPrefab::default()
        } else {
            PropsGroupPrefab::from_prefab(data)?
        };
        let mut props = Props::default();
        for (key, value) in data.data {
            if let Some(factory) = self.factories.get(&key) {
                (factory.1)(value, &mut props)?;
            } else {
                return Err(PrefabError::CouldNotDeserialize(format!(
                    "Could not find properties factory: {:?}",
                    key
                )));
            }
        }
        Ok(props)
    }
}

#[derive(Debug, Clone)]
pub enum PropsError {
    CouldNotReadData,
    HasNoDataOfType(String),
}

impl Prefab for PrefabValue {}

impl PropsData for PrefabValue
where
    Self: Clone,
{
    fn clone_props(&self) -> Box<dyn PropsData> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PropsGroupPrefab {
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub data: HashMap<String, PrefabValue>,
}

impl Prefab for PropsGroupPrefab {}

impl PropsData for PropsGroupPrefab
where
    Self: Clone,
{
    fn clone_props(&self) -> Box<dyn PropsData> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub trait PropsData: std::fmt::Debug + Send + Sync {
    fn clone_props(&self) -> Box<dyn PropsData>;
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn PropsData> {
    fn clone(&self) -> Self {
        self.clone_props()
    }
}

#[derive(Default, Clone)]
pub struct Props(HashMap<TypeId, Box<dyn PropsData>>);

impl Props {
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

    pub fn read_cloned_or_else<T, F>(&self, mut f: F) -> T
    where
        T: 'static + PropsData + Clone + Default,
        F: FnMut() -> T,
    {
        self.read_cloned().unwrap_or_else(|_| f())
    }

    pub fn write<T>(&mut self, data: T)
    where
        T: 'static + PropsData,
    {
        self.0
            .insert(TypeId::of::<T>(), Box::new(data) as Box<dyn PropsData>);
    }

    pub fn mutate<T, F>(&mut self, mut f: F)
    where
        T: 'static + PropsData,
        F: FnMut(&T) -> T,
    {
        if let Ok(data) = self.read() {
            let data = f(data);
            self.write(data);
        }
    }

    pub fn mutate_cloned<T, F>(&mut self, mut f: F)
    where
        T: 'static + PropsData + Clone,
        F: FnMut(&mut T),
    {
        if let Ok(data) = self.read::<T>() {
            let mut data = data.clone();
            f(&mut data);
            self.write(data);
        }
    }

    pub fn mutate_or_write<T, F, W>(&mut self, mut f: F, mut w: W)
    where
        T: 'static + PropsData,
        F: FnMut(&T) -> T,
        W: FnMut() -> T,
    {
        if let Ok(data) = self.read() {
            let data = f(data);
            self.write(data);
        } else {
            let data = w();
            self.write(data);
        }
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

    pub fn merge_from(&mut self, other: Self) {
        self.0.extend(other.into_inner());
    }

    pub(crate) fn into_inner(self) -> HashMap<TypeId, Box<dyn PropsData>> {
        self.0
    }
}

impl std::fmt::Debug for Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Props ")?;
        f.debug_set().entries(self.0.values()).finish()
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

        impl $crate::Prefab for $type_name {}
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
