use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use peace_resources::{resources::ts::SetUp, BorrowFail, Resources};
use serde::{Deserialize, Serialize, Serializer};

use crate::{FieldNameAndType, MappingFn, ParamsResolveError, ValueResolutionCtx};

/// Wrapper around a mapping function so that it can be serialized.
#[derive(Clone, Serialize, Deserialize)]
pub struct MappingFnImpl<T, F, Args> {
    /// This field's name within its parent struct.
    ///
    /// `None` if this is the top level value type.
    field_name: Option<String>,
    #[serde(
        default = "MappingFnImpl::<T, F, Args>::fn_map_none",
        skip_deserializing,
        serialize_with = "MappingFnImpl::<T, F, Args>::fn_map_serialize"
    )]
    fn_map: Option<F>,
    /// Marker.
    marker: PhantomData<(T, Args)>,
}

impl<T, F, Args> Debug for MappingFnImpl<T, F, Args>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MappingFnImpl")
            .field("field_name", &self.field_name)
            .field("fn_map", &Self::fn_map_stringify())
            .field("marker", &self.marker)
            .finish()
    }
}

impl<T, F, A0> MappingFnImpl<T, F, (A0,)>
where
    T: Clone + Debug + Send + Sync + 'static,
    F: Fn(&A0) -> Option<T> + Clone + Send + Sync + 'static,
    A0: Clone + Debug + Send + Sync + 'static,
{
    pub fn new(field_name: Option<String>, fn_map: F) -> Self {
        Self {
            fn_map: Some(fn_map),
            field_name,
            marker: PhantomData,
        }
    }

    pub fn map(
        &self,
        resources: &Resources<SetUp>,
        value_resolution_ctx: &mut ValueResolutionCtx,
    ) -> Result<T, ParamsResolveError> {
        if let Some(field_name) = self.field_name.as_deref() {
            value_resolution_ctx.push(FieldNameAndType::new(
                field_name.to_string(),
                std::any::type_name::<T>().to_string(),
            ));
        }
        let Some(fn_map) = self.fn_map.as_ref() else {
            panic!("`MappingFnImpl::map` called when `fn_map` is `None`.\n\
                This is a bug in the Peace framework.\n\
                \n\
                Type parameters are:\n\
                \n\
                * `T`: {t}\n\
                * `Args`: ({a0})\n\
                ",
                t = std::any::type_name::<T>(),
                a0 = std::any::type_name::<A0>(),
                );
        };

        let a0 = resources.try_borrow::<A0>().map(|a0| (a0,));

        match a0 {
            Ok((a0,)) => fn_map(&a0).ok_or(ParamsResolveError::FromMap {
                value_resolution_ctx: value_resolution_ctx.clone(),
                from_type_name: std::any::type_name::<A0>(),
            }),
            Err(borrow_fail) => match borrow_fail {
                BorrowFail::ValueNotFound => Err(ParamsResolveError::FromMap {
                    value_resolution_ctx: value_resolution_ctx.clone(),
                    from_type_name: std::any::type_name::<A0>(),
                }),
                BorrowFail::BorrowConflictImm | BorrowFail::BorrowConflictMut => {
                    Err(ParamsResolveError::FromMapBorrowConflict {
                        value_resolution_ctx: value_resolution_ctx.clone(),
                        from_type_name: std::any::type_name::<A0>(),
                    })
                }
            },
        }
    }

    pub fn try_map(
        &self,
        resources: &Resources<SetUp>,
        value_resolution_ctx: &mut ValueResolutionCtx,
    ) -> Result<Option<T>, ParamsResolveError> {
        if let Some(field_name) = self.field_name.as_deref() {
            value_resolution_ctx.push(FieldNameAndType::new(
                field_name.to_string(),
                std::any::type_name::<T>().to_string(),
            ));
        }
        let Some(fn_map) = self.fn_map.as_ref() else {
            panic!("`MappingFnImpl::try_map` called when `fn_map` is `None`.\n\
                This is a bug in the Peace framework.\n\
                \n\
                Type parameters are:\n\
                \n\
                * `T`: {t}\n\
                * `Args`: ({a0})\n\
                ",
                t = std::any::type_name::<T>(),
                a0 = std::any::type_name::<A0>(),
                );
        };
        match resources.try_borrow::<A0>() {
            Ok(u) => Ok(fn_map(&u)),
            Err(borrow_fail) => match borrow_fail {
                BorrowFail::ValueNotFound => Ok(None),
                BorrowFail::BorrowConflictImm | BorrowFail::BorrowConflictMut => {
                    Err(ParamsResolveError::FromMapBorrowConflict {
                        value_resolution_ctx: value_resolution_ctx.clone(),
                        from_type_name: std::any::type_name::<A0>(),
                    })
                }
            },
        }
    }
}

impl<T, F, Args> MappingFnImpl<T, F, Args> {
    fn fn_map_serialize<S>(_fn_map: &Option<F>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&Self::fn_map_stringify())
    }

    fn fn_map_stringify() -> String {
        format!(
            "Fn&{args} -> Option<{t}>",
            t = std::any::type_name::<T>(),
            args = std::any::type_name::<Args>(),
        )
    }

    fn fn_map_none() -> Option<F> {
        None
    }
}

impl<T, F, A0> MappingFn for MappingFnImpl<T, F, (A0,)>
where
    T: Clone + Debug + Send + Sync + 'static,
    F: Fn(&A0) -> Option<T> + Clone + Send + Sync + 'static,
    A0: Clone + Debug + Send + Sync + 'static,
{
    type Output = T;

    fn map(
        &self,
        resources: &Resources<SetUp>,
        value_resolution_ctx: &mut ValueResolutionCtx,
    ) -> Result<<Self as MappingFn>::Output, ParamsResolveError> {
        MappingFnImpl::map(self, resources, value_resolution_ctx)
    }

    fn try_map(
        &self,
        resources: &Resources<SetUp>,
        value_resolution_ctx: &mut ValueResolutionCtx,
    ) -> Result<Option<<Self as MappingFn>::Output>, ParamsResolveError> {
        MappingFnImpl::try_map(self, resources, value_resolution_ctx)
    }
}

impl<T, F, A0> From<(Option<String>, F)> for MappingFnImpl<T, F, (A0,)>
where
    T: Clone + Debug + Send + Sync + 'static,
    F: Fn(&A0) -> Option<T> + Clone + Send + Sync + 'static,
    A0: Clone + Debug + Send + Sync + 'static,
{
    fn from((field_name, f): (Option<String>, F)) -> Self {
        Self::new(field_name, f)
    }
}
