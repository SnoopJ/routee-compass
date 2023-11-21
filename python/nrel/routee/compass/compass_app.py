from __future__ import annotations

import json
import logging

from pathlib import Path
from typing import Any, Dict, List, Optional, Union
from nrel.routee.compass.routee_compass_py import (
    CompassAppWrapper,
)

import toml


Query = Dict[str, Any]
Result = List[Dict[str, Any]]


log = logging.getLogger(__name__)


class CompassApp:
    """
    The CompassApp holds everything needed to run a route query.
    """

    _app: CompassAppWrapper

    def __init__(self, app: CompassAppWrapper):
        self._app = app

    @classmethod
    def from_config_file(
        cls, config_file: Union[str, Path], output_file: Optional[str] = None
    ) -> CompassApp:
        """
        Build a CompassApp from a config file

        Args:
            config_file (Union[str, Path]): Path to the config file
            output_file (Optional[str]): Path to the output file. This overrides whatever is in the config file

        Returns:
            CompassApp: A CompassApp object

        Example:
            >>> from nrel.routee.compass import CompassApp
            >>> app = CompassApp.from_config_file("config.toml")
        """
        config_path = Path(config_file)
        if not config_path.is_file():
            raise ValueError(f"Config file {str(config_path)} does not exist")
        with open(config_path) as f:
            toml_config = toml.load(f)

        # inject the output file override into the to_disk plugin config
        if output_file is not None:
            override = False
            plugins = toml_config.get("plugin")
            if plugins is not None:
                output_plugins = plugins.get("output_plugins")
                if output_plugins is not None:
                    for plugin in output_plugins:
                        if plugin.get("type") == "to_disk":
                            override = True
                            plugin["output_file"] = output_file
            if not override:
                log.warning(
                    f"Output file override {output_file} was provided but "
                    "to_disk plugin was not found in config file"
                )

        toml_string = toml.dumps(toml_config)
        config_path_string = str(config_path.absolute())

        app = CompassAppWrapper._from_config_toml_string(
            toml_string, config_path_string
        )
        return cls(app)

    def run(self, query: Union[Query, List[Query]]) -> Result:
        """
        Run a query (or multiple queries) against the CompassApp

        Args:
            query (Union[Dict[str, Any], List[Dict[str, Any]]]): A query or list of queries to run

        Returns:
            List[Dict[str, Any]]: A list of results (or a single result if a single query was passed)

        Example:
            >>> from nrel.routee.compass import CompassApp
            >>> app = CompassApp.from_config_file("config.toml")
            >>> query = {
                    "origin_name": "NREL",
                    "destination_name": "Comrade Brewing Company",
                    "origin_x": -105.1710052,
                    "origin_y": 39.7402804,
                    "destination_x": -104.9009913,
                    "destination_y": 39.6757025
                }

            >>> result = app.run(query)

        """
        if isinstance(query, dict):
            queries = [query]
            single_query = True
        elif isinstance(query, list):
            queries = query
            single_query = False
        else:
            raise ValueError(
                f"Query must be a dict or list of dicts, not {type(query)}"
            )

        queries_json = list(map(json.dumps, queries))

        results_json: List[str] = self._app._run_queries(queries_json)

        results = list(map(json.loads, results_json))
        if single_query and len(results) == 1:
            return results[0]
        return results
