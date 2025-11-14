use crate::parser::DeriveInput;
use proc_macro::TokenStream;

fn format_generics(input: &DeriveInput) -> (String, String) {
    let lifetimes = if input.generics.lifetimes.is_empty() {
        String::new()
    } else {
        input
            .generics
            .lifetimes
            .iter()
            .map(|l| format!("'{}, ", l))
            .collect::<Vec<_>>()
            .join("")
    };

    let type_params = if input.generics.type_params_with_bounds.is_empty() {
        String::new()
    } else {
        input.generics.type_params_with_bounds.join(", ")
    };

    (lifetimes, type_params)
}

pub(crate) fn impl_mod_macro(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let ty_generics = input.generics.type_generics();
    let (lifetimes, type_params) = format_generics(&input);

    let where_clause = input
        .generics
        .where_clause
        .as_ref()
        .map(|w| format!("{} ", w))
        .unwrap_or_default();

    let code = format!(
        r"impl<{lifetimes}{type_params}> DatagramL<'_> for {name}{ty_generics} {where_clause} {{
            type G = ModulationOperationGenerator;
            type Error = ModulationError;
            fn operation_generator_with_finite_loop(self, _: &Geometry, _: &Environment, _: &DeviceMask, segment: Segment, transition_params: transition_mode::TransitionModeParams, rep: u16) -> Result<Self::G, Self::Error> {{
                let config = <Self as Modulation>::sampling_config(&self);
                let g = self.calc()?;
                Ok(Self::G {{ g: std::sync::Arc::new(g), config, rep, segment, transition_params, }})
            }}
            fn option(&self) -> DatagramOption {{ DatagramOption::default() }}
        }}
        impl<{lifetimes}{type_params}> Inspectable<'_> for {name}{ty_generics} {where_clause} {{
            type Result = ModulationInspectionResult;
            fn inspect(self, geometry: &Geometry, _: &Environment, filter: &DeviceMask) -> Result<InspectionResult<Self::Result>, ModulationError> {{
                let sampling_config = self.sampling_config();
                sampling_config.divide()?;
                let data = self.calc()?;
                Ok(InspectionResult::new(geometry, filter, |_| ModulationInspectionResult {{ data: data.clone(), config: sampling_config, }}))
            }}
        }}
        impl<{lifetimes}{type_params}> internal::HasSegment<transition_mode::Immediate> for {name}{ty_generics} {where_clause}{{}}
        impl<{lifetimes}{type_params}> internal::HasSegment<transition_mode::Ext> for {name}{ty_generics} {where_clause}{{}}
        impl<{lifetimes}{type_params}> internal::HasSegment<transition_mode::Later> for {name}{ty_generics} {where_clause}{{}}
        impl<{lifetimes}{type_params}> internal::HasFiniteLoop<transition_mode::SyncIdx> for {name}{ty_generics} {where_clause}{{}}
        impl<{lifetimes}{type_params}> internal::HasFiniteLoop<transition_mode::SysTime> for {name}{ty_generics} {where_clause}{{}}
        impl<{lifetimes}{type_params}> internal::HasFiniteLoop<transition_mode::GPIO> for {name}{ty_generics} {where_clause}{{}}
        impl<{lifetimes}{type_params}> internal::HasFiniteLoop<transition_mode::Later> for {name}{ty_generics} {where_clause}{{}}",
    );

    code.parse().unwrap()
}
