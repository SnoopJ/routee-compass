# routee-compass

A routing engine that considers energy weights on edges of a graph for particular vehicle types - built for integration with RouteE.

## setup

### from pip

```bash
pip install nrel.routee.compass --extra-index-url=https://github.nrel.gov/pages/MBAP/mbap-pypi/
```

### from source

```bash
git clone https://github.nrel.gov/MBAP/routee-compass.git
cd routee-compass

pip install .
```

### rust extension

If you want to use the rust extension you'll need to install rust.
One way to do this is to use conda

```bash
conda create -n routee-compass python=3.10 rust
conda activate routee-compass
```

Then, you'll need to get the rust build tool maturin

```bash
pip install maturin
```

Then, you can build the rust extension

```bash
cd rust/
maturin develop --release
```

### get a road network

We support the tomtom current road network.

```bash
cd scripts
python download_road_map.py <path/to/polygon.geojson> <my-road-network.json>
```

note: you'll need access to the trolley postgres server.

## start routing

Once you have a road network file downloaded you can start computing least energy routes.

Here's a sample workflow for loading the road network and finding the least energy path:

```python
from nrel.routee.compass.compass_map import compute_energy
from nrel.routee.compass.rotuee_model_collection import RouteeModelCollection

from mappymatch.constructs.coordinate import Coordinate
from mappymatch.maps.nx.nx_map import NxMap

road_network = NxMap.from_file("path/to/my/tomtom_road_network.json")

routee_models = RouteeModelCollection()

compute_energy(road_network, routee_models)

origin = Coordinate.from_lat_lon(lat=39.00, lon=-104.00)
destination = Coordinate.from_lat_lon(lat=39.10, lon=-104.10)

shortest_energy_route = road_network.shortest_path(origin, destination, weight="Electric")
```

The road network will compute energy over the whole graph so it could take some time if the graph is large.

Note that routee-compass comes with two default routee-powertrain models "Gasoline" and "Electric".

If you want to use your own routee models you can do so like this:

```python
from nrel.routee.compass.compass_map import compute_energy
from nrel.routee.compass.rotuee_model_collection import RouteeModelCollection

from mappymatch.constructs.coordinate import Coordinate
from mappymatch.maps.nx.nx_map import NxMap

my_routee_models = {
    "Tesla": "path/to/tesla_model.json",
    "Ferrari": "path/to/ferrari_model.json",
}
routee_models = RouteeModelCollection(my_routee_models)

road_network = NxMap.from_file("path/to/my/tomtom_road_network.json")

compute_energy(road_network, routee_models)

origin = Coordinate(lat=39.00, lon=-104.00)
destination = Coordinate(lat=39.10, lon=-104.10)

tesla_shortest_energy_route = road_network.shortest_path(origin, destination, weight="Tesla")
ferrari_shortest_energy_route = road_network.shortest_path(origin, destination, weight="Ferrari")
```
