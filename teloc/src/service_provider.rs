use crate::container_elem::{
    ConvertContainerElem, Init, InstanceContainerElem, SingletonContainerElem,
    TransientContainerElem,
};
use crate::scope::{InitScope, InitScoped, ScopedContainerElem, ScopedInstanceContainerElem};
use crate::Scope;
use frunk::hlist::{HList, Selector};
use frunk::{HCons, HNil};
use std::marker::PhantomData;

pub struct ServiceProvider<Dependencies, Scoped, ScopedI> {
    dependencies: Dependencies,
    scoped_i: PhantomData<ScopedI>,
    scoped: PhantomData<Scoped>,
}

impl ServiceProvider<HNil, HNil, HNil> {
    pub fn new() -> Self {
        ServiceProvider {
            dependencies: HNil,
            scoped_i: PhantomData,
            scoped: PhantomData,
        }
    }
}

impl Default for ServiceProvider<HNil, HNil, HNil> {
    fn default() -> Self {
        Self::new()
    }
}

type ContainerAddConvertElem<T, U, H, S, SI> =
    ServiceProvider<HCons<ConvertContainerElem<TransientContainerElem<T>, T, U>, H>, S, SI>;

impl<H: HList, S, SI> ServiceProvider<H, S, SI> {
    /// Method used primary for internal actions. In common usage you don't need to use it. It add dependencies to the store. You need
    /// to put in first generic parameter some `ContainerElem` type.
    /// Usage:
    ///
    /// ```
    /// use teloc::*;
    /// use teloc::container_elem::TransientContainerElem;
    ///
    /// struct Service {
    ///     data: i32,
    /// }
    ///
    /// let sp = ServiceProvider::new()
    ///     ._add::<TransientContainerElem<Service>>(());
    /// ```
    pub fn _add<'a, T: Init>(self, data: T::Data) -> ServiceProvider<HCons<T, H>, S, SI> {
        let ServiceProvider { dependencies, .. } = self;
        ServiceProvider {
            dependencies: dependencies.prepend(T::init(data)),
            scoped_i: PhantomData,
            scoped: PhantomData,
        }
    }
    /// Add dependency with the `Transient` lifetime. Transient services will be created each time
    /// when it called. Use this lifetime for lightweight stateless services.
    ///
    /// Can be resolved only by ownership.
    ///
    /// Usage:
    /// ```
    /// use teloc::*;
    /// use uuid::Uuid;
    ///
    /// struct Service { uuid: Uuid }
    /// #[inject]
    /// impl Service {
    ///     fn new() -> Self { Self { uuid: Uuid::new_v4() } }
    /// }
    ///
    /// let sp = ServiceProvider::new()
    ///     .add_transient::<Service>();
    ///
    /// let s1: Service = sp.resolve();
    /// let s2: Service = sp.resolve();
    ///
    /// assert_ne!(s1.uuid, s2.uuid);
    /// ```
    pub fn add_transient<T>(self) -> ServiceProvider<HCons<TransientContainerElem<T>, H>, S, SI>
    where
        TransientContainerElem<T>: Init<Data = ()>,
    {
        self._add::<TransientContainerElem<T>>(())
    }
    #[inline]
    pub fn add_scoped<T>(self) -> ServiceProvider<H, HCons<ScopedContainerElem<T>, S>, SI> {
        let ServiceProvider { dependencies, .. } = self;
        ServiceProvider {
            dependencies,
            scoped_i: PhantomData,
            scoped: PhantomData,
        }
    }
    #[inline]
    pub fn add_scoped_i<T>(
        self,
    ) -> ServiceProvider<H, S, HCons<ScopedInstanceContainerElem<T>, SI>> {
        let ServiceProvider { dependencies, .. } = self;
        ServiceProvider {
            dependencies,
            scoped_i: PhantomData,
            scoped: PhantomData,
        }
    }
    pub fn add_singleton<T>(self) -> ServiceProvider<HCons<SingletonContainerElem<T>, H>, S, SI>
    where
        SingletonContainerElem<T>: Init<Data = ()>,
    {
        self._add::<SingletonContainerElem<T>>(())
    }
    pub fn add_instance<T>(
        self,
        data: T,
    ) -> ServiceProvider<HCons<InstanceContainerElem<T>, H>, S, SI>
    where
        InstanceContainerElem<T>: Init<Data = T>,
    {
        self._add::<InstanceContainerElem<T>>(data)
    }
    pub fn add_transient_<U, T>(self) -> ContainerAddConvertElem<T, U, H, S, SI>
    where
        T: Into<U>,
        ConvertContainerElem<TransientContainerElem<T>, T, U>: Init<Data = ()>,
        TransientContainerElem<T>: Init<Data = ()>,
    {
        self._add::<ConvertContainerElem<TransientContainerElem<T>, T, U>>(())
    }
}

impl<'a, H, S, SI> ServiceProvider<H, S, SI>
where
    S: InitScoped,
{
    pub fn scope(&self, si: SI) -> Scope<Self, S, SI> {
        Scope::new(self, si)
    }
}

impl<H, S, SI> ServiceProvider<H, S, SI> {
    pub fn dependencies(&self) -> &H {
        &self.dependencies
    }
}

impl<H, S, SI, T, Index> Selector<T, Index> for ServiceProvider<H, S, SI>
where
    H: Selector<T, Index>,
{
    fn get(&self) -> &T {
        self.dependencies().get()
    }

    fn get_mut(&mut self) -> &mut T {
        self.dependencies.get_mut()
    }
}
