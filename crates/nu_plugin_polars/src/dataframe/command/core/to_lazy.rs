use crate::{dataframe::values::NuSchema, values::CustomValueSupport, Cacheable, PolarsPlugin};

use crate::values::{NuDataFrame, NuLazyFrame};

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    record, Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};

#[derive(Clone)]
pub struct ToLazyFrame;

impl PluginCommand for ToLazyFrame {
    type Plugin = PolarsPlugin;

    fn name(&self) -> &str {
        "polars into-lazy"
    }

    fn description(&self) -> &str {
        "Converts a dataframe into a lazy dataframe."
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .named(
                "schema",
                SyntaxShape::Record(vec![]),
                r#"Polars Schema in format [{name: str}]."#,
                Some('s'),
            )
            .input_output_type(Type::Any, Type::Custom("dataframe".into()))
            .category(Category::Custom("lazyframe".into()))
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Takes a table and creates a lazyframe",
            example: "[[a b];[1 2] [3 4]] | polars into-lazy",
            result: None,
        },
        Example {
            description: "Takes a table, creates a lazyframe, assigns column 'b' type str, displays the schema",
            example: "[[a b];[1 2] [3 4]] | polars into-lazy --schema {b: str} | polars schema",
            result: Some(Value::test_record(record! {"b" => Value::test_string("str")})),
        },
        ]
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let maybe_schema = call
            .get_flag("schema")?
            .map(|schema| NuSchema::try_from_value(plugin, &schema))
            .transpose()?;

        let df = NuDataFrame::try_from_iter(plugin, input.into_iter(), maybe_schema)?;
        let mut lazy = NuLazyFrame::from_dataframe(df);
        // We don't want this converted back to an eager dataframe at some point
        lazy.from_eager = false;
        Ok(PipelineData::Value(
            lazy.cache(plugin, engine, call.head)?.into_value(call.head),
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::test::test_polars_plugin_command;
    use std::sync::Arc;

    use nu_plugin_test_support::PluginTest;
    use nu_protocol::{ShellError, Span};

    use super::*;

    #[test]
    fn test_to_lazy() -> Result<(), ShellError> {
        let plugin: Arc<PolarsPlugin> = PolarsPlugin::new_test_mode()?.into();
        let mut plugin_test = PluginTest::new("polars", Arc::clone(&plugin))?;
        let pipeline_data = plugin_test.eval("[[a b]; [6 2] [1 4] [4 1]] | polars into-lazy")?;
        let value = pipeline_data.into_value(Span::test_data())?;
        let df = NuLazyFrame::try_from_value(&plugin, &value)?;
        assert!(!df.from_eager);
        Ok(())
    }

    #[test]
    fn test_examples() -> Result<(), ShellError> {
        test_polars_plugin_command(&ToLazyFrame)
    }
}
