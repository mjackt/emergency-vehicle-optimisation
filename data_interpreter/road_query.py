import json
import requests

query = """
[out:json];
area[name="Devon"]->.search;
area["name"="England"]->.uk;

( way["highway"~"motorway|trunk|primary|secondary|tertiary|unclassified|residential|service|living_street|track|road"](area.uk)(area.search);
);

out body;
>;
out skel qt;
"""
url = "https://overpass-api.de/api/interpreter"
response = requests.post(url, data=query)
data = response.json()

with open('input_data/devon/osm.json', 'w') as f:
    json.dump(data, f)