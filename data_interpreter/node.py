import math
import json
import json_fix

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
        for node in self.outs:
            out_list.append(node[0].id)

        return f"\n {self.id}\n{self.location}\n{out_list}\n\n"
    
    def __repr__(self):
        out_list = []
        for node in self.outs:
            out_list.append(node[0].id)

        return f"\n {self.id}\n{self.location}\n{out_list}\n\n"

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

    def to_dict(self):
        # Create a dictionary representation of the Node, including only the relevant data
        return {
            "outs": [{"id": out_node[0].id, "cost": out_node[1]} for out_node in self.outs]
        }      



def calc_distance(lat1, lon1, lat2, lon2):
    R = 6378.137 #Earth radius in km

    dLat = lat2 * math.pi / 180 - lat1 * math.pi / 180
    dLon = lon2 * math.pi / 180 - lon1 * math.pi / 180

    a = math.sin(dLat/2) * math.sin(dLat/2) + \
    math.cos(lat1 * math.pi / 180) * math.cos(lat2 * math.pi / 180) * \
    math.sin(dLon/2) * math.sin(dLon/2)

    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1-a))
    d = R * c
    return d * 1000 #metres