use crate::parser::DeriveInput;
use proc_macro::TokenStream;

fn format_generics(input: &DeriveInput) -> (String, String, String, String) {
    let lifetimes: Vec<_> = input
        .generics
        .lifetimes
        .iter()
        .filter(|l| l.as_str() != "geo")
        .collect();

    let lifetimes = if lifetimes.is_empty() {
        String::new()
    } else {
        lifetimes
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

    let ty_generics = input.generics.type_generics();

    let where_clause = if let Some(ref w) = input.generics.where_clause {
        format!("{} Self: Gain<'geo>", w)
    } else {
        "where Self: Gain<'geo>".to_string()
    };

    (lifetimes, type_params, ty_generics, where_clause)
}

pub(crate) fn impl_gain_macro(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (lifetimes, type_params, ty_generics, where_clause) = format_generics(&input);

    let code = format!(
        r"impl<'geo, {lifetimes}{type_params}> DatagramS<'geo> for {name}{ty_generics} {where_clause} {{
            type G = GainOperationGenerator<'geo, <Self as Gain<'geo>>::G>;
            type Error = GainError;
            fn operation_generator_with_segment(self, geometry: &'geo Geometry, env: &Environment, filter: &DeviceMask, segment: Segment, transition_params: transition_mode::TransitionModeParams) -> Result<Self::G, Self::Error> {{
                Self::G::new(self, geometry, env, filter, segment, transition_params)
            }}
            fn option(&self) -> DatagramOption {{
                DatagramOption {{ parallel_threshold: std::thread::available_parallelism().map(std::num::NonZeroUsize::get).unwrap_or(8), ..DatagramOption::default() }}
            }}
        }}
        impl<'geo, {lifetimes}{type_params}> Inspectable<'geo> for {name}{ty_generics} {where_clause} {{
            type Result = GainInspectionResult;
            fn inspect(self, geometry: &'geo Geometry, env: &Environment, filter: &DeviceMask) -> Result<InspectionResult<GainInspectionResult>, GainError> {{
                let mut g = self.init(geometry, env, &TransducerMask::from(filter))?;
                Ok(InspectionResult::new(geometry, filter, |dev| GainInspectionResult {{ data: {{ let d = g.generate(dev); dev.iter().map(|tr| d.calc(tr)).collect::<Vec<_>>() }}, }}))
            }}
        }}
        impl<'geo, {lifetimes}{type_params}> internal::HasSegment<transition_mode::Immediate> for {name}{ty_generics} {where_clause} {{}}
        impl<'geo, {lifetimes}{type_params}> internal::HasSegment<transition_mode::Later> for {name}{ty_generics} {where_clause} {{}}",
    );

    code.parse().unwrap()
}
