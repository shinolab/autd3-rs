use crate::parser::DeriveInput;
use proc_macro::TokenStream;

fn format_generics(input: &DeriveInput) -> (String, String, String, String) {
    // Filter out 'geo lifetime for the impl generics
    let lifetimes: Vec<_> = input
        .generics
        .lifetimes
        .iter()
        .filter(|l| l.as_str() != "geo")
        .collect();

    let lifetimes_str = if lifetimes.is_empty() {
        String::new()
    } else {
        lifetimes
            .iter()
            .map(|l| format!("'{}", l))
            .collect::<Vec<_>>()
            .join(", ")
            + ", "
    };

    let type_params_str = if input.generics.type_params_with_bounds.is_empty() {
        String::new()
    } else {
        input.generics.type_params_with_bounds.join(", ") + ", "
    };

    let ty_generics = input.generics.type_generics();

    let where_clause = if let Some(ref w) = input.generics.where_clause {
        format!("{} Self: Gain<'geo>,", w)
    } else {
        "where Self: Gain<'geo>,".to_string()
    };

    (lifetimes_str, type_params_str, ty_generics, where_clause)
}

pub(crate) fn impl_gain_macro(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (lifetimes_str, type_params_str, ty_generics, where_clause) = format_generics(&input);

    let code = format!(
        "impl<'geo, {lifetimes}{type_params}> DatagramS<'geo> for {name}{ty_generics} {where_clause} {{ \
            type G = GainOperationGenerator<'geo, <Self as Gain<'geo>>::G>; \
            type Error = GainError; \
            fn operation_generator_with_segment(self, geometry: &'geo Geometry, env: &Environment, filter: &DeviceMask, segment: Segment, transition_params: transition_mode::TransitionModeParams) -> Result<Self::G, Self::Error> {{ \
                Self::G::new(self, geometry, env, filter, segment, transition_params) \
            }} \
            fn option(&self) -> DatagramOption {{ \
                DatagramOption {{ parallel_threshold: std::thread::available_parallelism().map(std::num::NonZeroUsize::get).unwrap_or(8), ..DatagramOption::default() }} \
            }} \
        }} \
        impl<'geo, {lifetimes}{type_params}> Inspectable<'geo> for {name}{ty_generics} {where_clause} {{ \
            type Result = GainInspectionResult; \
            fn inspect(self, geometry: &'geo Geometry, env: &Environment, filter: &DeviceMask) -> Result<InspectionResult<GainInspectionResult>, GainError> {{ \
                let mut g = self.init(geometry, env, &TransducerMask::from(filter))?; \
                Ok(InspectionResult::new(geometry, filter, |dev| GainInspectionResult {{ data: {{ let d = g.generate(dev); dev.iter().map(|tr| d.calc(tr)).collect::<Vec<_>>() }}, }})) \
            }} \
        }} \
        impl<'geo, {lifetimes}{type_params}> internal::HasSegment<transition_mode::Immediate> for {name}{ty_generics} {where_clause} {{}} \
        impl<'geo, {lifetimes}{type_params}> internal::HasSegment<transition_mode::Later> for {name}{ty_generics} {where_clause} {{}}",
        lifetimes = lifetimes_str,
        type_params = type_params_str,
        name = name,
        ty_generics = ty_generics,
        where_clause = where_clause
    );

    code.parse().unwrap()
}
