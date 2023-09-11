use crate::plugin::input::input_field::InputField;
use crate::plugin::input::input_json_extensions::InputJsonExtensions;
use crate::plugin::input::input_plugin::InputPlugin;
use crate::plugin::plugin_error::PluginError as E;
use compass_core::util::multiset::MultiSet;

/// Builds an input plugin that duplicates queries if array-valued fields are present
/// by stepping through each combination of value
pub struct GridSearchPlugin {}

impl InputPlugin for GridSearchPlugin {
    fn process(&self, input: &serde_json::Value) -> Result<Vec<serde_json::Value>, E> {
        match input.get_grid_search() {
            None => Ok(vec![input.clone()]),
            Some(grid_search_input) => {
                let map = grid_search_input
                    .as_object()
                    .ok_or(E::UnexpectedQueryStructure(format!("{:?}", input)))?;
                let mut keys: Vec<String> = vec![];
                let mut multiset_input: Vec<Vec<serde_json::Value>> = vec![];
                let mut multiset_indices: Vec<Vec<usize>> = vec![];
                for (k, v) in map {
                    match v {
                        serde_json::Value::Array(values) => {
                            keys.push(k.to_string());
                            multiset_input.push(values.to_vec());
                            let indices = (0..values.len()).collect();
                            multiset_indices.push(indices);
                        }
                        _ => {}
                    }
                }
                // for each combination, copy the grid search values into a fresh
                // copy of the source (minus the "grid_search" key)
                // let remove_key = InputField::GridSearch.to_str();
                let mut initial_map = input
                    .as_object()
                    .ok_or(E::UnexpectedQueryStructure(format!("{:?}", input)))?
                    .clone();
                initial_map.remove(InputField::GridSearch.to_str());
                let initial = serde_json::json!(initial_map);
                let multiset = MultiSet::from(&multiset_indices);
                let result: Vec<serde_json::Value> = multiset
                    .into_iter()
                    .map(|combination| {
                        let mut instance = initial.clone();
                        for (set_idx, (key, val_idx)) in
                            keys.iter().zip(combination.iter()).enumerate()
                        {
                            instance[key] = multiset_input[set_idx][*val_idx].clone();
                        }
                        instance
                    })
                    .collect();

                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::GridSearchPlugin;
    use crate::plugin::input::input_plugin::InputPlugin;

    #[test]
    fn test_grid_search_empty_parent_object() {
        let input = serde_json::json!({
            "grid_search": {
                "bar": ["a", "b", "c"],
                "foo": [1.2, 3.4]
            }
        });
        let plugin = GridSearchPlugin {};
        let result = plugin
            .process(&input)
            .unwrap()
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<String>, serde_json::Error>>()
            .unwrap();
        let expected = vec![
            String::from("{\"bar\":\"a\",\"foo\":1.2}"),
            String::from("{\"bar\":\"b\",\"foo\":1.2}"),
            String::from("{\"bar\":\"c\",\"foo\":1.2}"),
            String::from("{\"bar\":\"a\",\"foo\":3.4}"),
            String::from("{\"bar\":\"b\",\"foo\":3.4}"),
            String::from("{\"bar\":\"c\",\"foo\":3.4}"),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_grid_search_persisted_parent_keys() {
        let input = serde_json::json!({
            "persistent": "key",
            "grid_search": {
                "bar": ["a", "b", "c"],
                "foo": [1.2, 3.4]
            }
        });
        let plugin = GridSearchPlugin {};
        let result = plugin
            .process(&input)
            .unwrap()
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<String>, serde_json::Error>>()
            .unwrap();
        let expected = vec![
            String::from("{\"bar\":\"a\",\"foo\":1.2,\"persistent\":\"key\"}"),
            String::from("{\"bar\":\"b\",\"foo\":1.2,\"persistent\":\"key\"}"),
            String::from("{\"bar\":\"c\",\"foo\":1.2,\"persistent\":\"key\"}"),
            String::from("{\"bar\":\"a\",\"foo\":3.4,\"persistent\":\"key\"}"),
            String::from("{\"bar\":\"b\",\"foo\":3.4,\"persistent\":\"key\"}"),
            String::from("{\"bar\":\"c\",\"foo\":3.4,\"persistent\":\"key\"}"),
        ];
        assert_eq!(result, expected);
    }
}
