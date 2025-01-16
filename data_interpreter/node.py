import math
import json
import json_fix
import numpy as np

class Node:
#List of out nodes
    def __init__(self, id, location, incid_in_year, police = False):
        #id provided by OSM
        self.id: int = id
        #All nodes reachable from current node. != to all nodes that can reach this node due to one ways
        self.outs: list[(Node, float)] = []

        self.ins: list[Node] = []
        #Coords of node
        self.location: tuple[float,float] = location
        #Number of incdients per year (Can be floats)
        self.incid_in_year: float = incid_in_year

        self.police: bool = police

    def __str__(self):
        out_list = []
        in_list = []
        for node in self.outs:
            out_list.append(node[0].id)

        for node in self.ins:
            in_list.append(node.id)

        return f"\n {self.id}\n{self.location}\nOuts: {out_list}\nIns: {in_list}\n\n"
    
    def __repr__(self):
        out_list = []
        in_list = []
        for node in self.outs:
            out_list.append(node[0].id)

        for node in self.ins:
            in_list.append(node.id)

        return f"\n {self.id}\n{self.location}\nOuts: {out_list}\nIns: {in_list}\n\n"

    def add_out(self, node_to_add, speed):
        #Speed in m/s 

        distance = calc_distance(self.location[0], self.location[1], node_to_add.location[0], node_to_add.location[1])

        #Time in secs to go from current node to out node
        cost = distance / speed

        added = False

        for i in range(len(self.outs)):
            if cost < self.outs[i][1]:
                self.outs.insert(i,(node_to_add, cost))
                added = True
                break
            
        if added == False:
            self.outs.append((node_to_add, cost))

    def add_in(self, node_to_add):
        if node_to_add not in self.ins:
            self.ins.append(node_to_add)

    def to_dict(self):
        # Create a dictionary representation of the Node, including only the relevant data
        return {
            "outs": [{"id": out_node[0].id, "cost": out_node[1]} for out_node in self.outs]
        }   

    def add_incident(self, incid):
        self.incid_in_year += incid   


DEG_TO_RAD = np.pi / 180
def calc_distance(lat1, lon1, lat2, lon2):
    R = 6378.137 #Earth radius in km

    dLat = (lat2 - lat1) * DEG_TO_RAD
    dLon = (lon2 - lon1) * DEG_TO_RAD

    a = np.sin(dLat / 2) ** 2 + np.cos(lat1 * DEG_TO_RAD) * np.cos(lat2 * DEG_TO_RAD) * np.sin(dLon / 2) ** 2

    c = 2 * np.arctan2(np.sqrt(a), np.sqrt(1 - a))
    return R * c * 1000  # metres