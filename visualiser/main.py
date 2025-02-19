import json
import sys
import folium
import csv

SF = 15

class Storage:
    def __init__(self, name, lat, lon):
        self.name = name
        self.lat = lat
        self.lon = lon
        self.max = 0
        self.min = sys.maxsize
        self.sum = 0


if __name__=="__main__":
    m = folium.Map(location=(50.67, -4.125), zoom_start=8)

    with open('results.csv', newline='') as f:
        reader = csv.reader(f)
        rows = list(reader)

    with open('police.json', 'r') as file:
        police = json.load(file)

    storage_list = []
    #First iterate over first row to get names and use police.json for lat and lon
    for i in range(len(rows[0])):
        if police['elements'][i]['type'] == "node":
            lat, lon = 0,0
            lat = police['elements'][i]['lat']
            lon = police['elements'][i]['lon']

        elif police['elements'][i]['type'] == "way":
            lat, lon = 0,0
            node_id = police['elements'][i]['nodes'][0]
            for element in police['elements']:
                if element['type'] == 'node' and element['id'] == node_id:
                    lat: float = element['lat']
                    lon: float = element['lon']
        else:#relation
            lat, lon = 0,0
            way_id = police['elements'][i]['members'][0]['ref']
            for element in police['elements']:
                if element['type'] == 'way' and element['id'] == way_id:
                    first_node_id = element['nodes'][0]
            for element in police['elements']:
                if element['type'] == 'node' and element['id'] == first_node_id:
                    lat: float = element['lat']
                    lon: float = element['lon']

        storage_list.append(Storage(rows[0][i], lat, lon))
        

    #Iterate over data rows
    for i in range(1, len(rows)):
        for j in range(len(rows[i])):
            value=int(rows[i][j])
            storage_list[j].sum += value
            if value < storage_list[j].min:
                storage_list[j].min = value
            elif value > storage_list[j].max:
                storage_list[j].max = value

    bubble_layer = folium.FeatureGroup(name="Bubble Map", show=True)
    min_layer = folium.FeatureGroup(name="Min Values", show=False)
    max_layer = folium.FeatureGroup(name="Max Values", show=False)


    for storage in storage_list:
        avg = storage.sum/(len(rows)-1)
        if avg > 0:
            folium.CircleMarker(
                location=[storage.lat, storage.lon],
                radius=avg * SF,  #Average num
                color="blue",
                fill=True,
                fill_color="blue",
                fill_opacity=0.5,
                popup=f"""<b>{storage.name}</b> 
                        <br>
                        <span style="display:inline-block; width:80px;">Average: {avg}</span>
                        <br>
                        <span style="display:inline-block; width:80px;">Min: {storage.min}</span>
                        <br>
                        <span style="display:inline-block; width:80px;">Max: {storage.max}</span>""",
                ).add_to(bubble_layer)
        else:
            folium.Marker(
                location=[storage.lat, storage.lon],
                icon=folium.DivIcon(html=f'<div style="color:red; font-size:12px;">✖</div>'),
                popup=f"{storage.name}"
            ).add_to(bubble_layer)

        if storage.min > 0:
            folium.CircleMarker(
                location=[storage.lat, storage.lon],
                radius=storage.min * SF, 
                color="green",
                fill=True,
                fill_color="green",
                fill_opacity=0.0,
                weight = 1
            ).add_to(min_layer)

        if storage.max > 0:
            folium.CircleMarker(
                location=[storage.lat, storage.lon],
                radius=storage.max * SF, 
                color="red",
                fill=True,
                fill_color="red",
                fill_opacity=0.0,
                weight = 1
            ).add_to(max_layer)

    m.add_child(bubble_layer)
    m.add_child(min_layer)
    m.add_child(max_layer)

    folium.LayerControl().add_to(m)

    m.save("index.html")