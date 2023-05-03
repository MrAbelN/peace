use peace_resources::{resources::ts::SetUp, type_reg::untagged::DataType, Resources};
use serde::{Serialize, Serializer};

use crate::ParamsResolveError;

/// Type erased mapping function.
///
/// This is used by Peace to hold type-erased mapping functions, and is not
/// intended to be implemented by users or implementors.
pub trait MappingFn: DataType {
    /// Type that is output by the function.
    type Output;

    /// Maps data in resources to the output type.
    ///
    /// The data being accessed is defined by the implementation of this
    /// function.
    ///
    /// # Parameters
    ///
    /// * `resources`: Resources to resolve values from.
    /// * `params_type_name_fn`: Function to retrieve the params type name.
    /// * `field_name_fn`: Function to retrieve the field name.
    fn map(
        &self,
        resources: &Resources<SetUp>,
        params_type_name_fn: fn() -> &'static str,
        field_name: &'static str,
    ) -> Result<Self::Output, ParamsResolveError>;

    /// Maps data in resources to the output type.
    ///
    /// The data being accessed is defined by the implementation of this
    /// function.
    ///
    /// # Parameters
    ///
    /// * `resources`: Resources to resolve values from.
    /// * `params_type_name_fn`: Function to retrieve the params type name.
    /// * `field_name_fn`: Function to retrieve the field name.
    fn try_map(
        &self,
        resources: &Resources<SetUp>,
        params_type_name_fn: fn() -> &'static str,
        field_name: &'static str,
    ) -> Result<Option<Self::Output>, ParamsResolveError>;
}

impl<T> Clone for Box<dyn MappingFn<Output = T>> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

impl<'a, T> Serialize for dyn MappingFn<Output = T> + 'a {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Sadly the following doesn't work, it says the lifetime of:
        // `&'1 self` must outlive `'static`
        //
        // let data_type: &(dyn DataType + 'a) = &self;
        // Serialize::serialize(data_type, serializer)

        // so we have to depend on `erased_serde` directly
        erased_serde::serialize(self, serializer)
    }
}