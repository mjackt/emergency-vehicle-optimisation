import math
import json
import json_fix
import numpy as np

class Node:
    """
    Class to represent the nodes in a graph.

    Args:
        id (int): The unique ID of the node. In this program's context it will be the ID provided by OSM.
        location (tuple[float, float]): The coordinates of the node. (Lat, Lon) in decimal degree format.
        incid_in_year (float): The number of incidents that will occur at this node over a year. Doesn't need to be whole numbers.
        police (bool): Shows whether the node is a police station or just a regular node.

    Attributes:
        id (int): The unique ID of the node. In this program's context it will be the ID provided by OSM.
        outs (list[(Node, float)]): The nodes reachable by leaving this node. Stored as a list of tuples with format (Node_obj, cost_to_node). Sorted by min cost first.
        ins (list[Node]): A list of Node objects that go into this node. Will not be the same as outs because of one way systems.
        location (tuple[float, float]): The coordinates of the node. (Lat, Lon) in decimal degree format.
        incid_in_year (float): The number of incidents that will occur at this node over a year. Doesn't need to be whole numbers.
        police (bool): Shows whether the node is a police station or just a regular node.
    """
    def __init__(self, id: int, location: tuple[float,float], incid_in_year: float, police: bool = False):
        self.id: int = id
        self.outs: list[(Node, float)] = []
        self.ins: list[Node] = []
        self.location: tuple[float,float] = location
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

    def add_out(self, node_to_add, speed: float):
        """
        Creates an out from the node to another node and adds it to this node's list of outs.
        Calculates cost of travel based on a speed value.
        Out is inserted into outs based on cost.

        Args:
            node_to_add (Node): The node object to be added as an out.
            speed (float): The speed that will be travelled at on this edge (m/s).
        """
        distance: float = calc_distance(self.location[0], self.location[1], node_to_add.location[0], node_to_add.location[1])

        #Time in secs to go from current node to out node
        cost: float = distance / speed

        added: bool = False

        #Insertion loop based on travel cost. Min cost at index 0.
        for i in range(len(self.outs)):
            if cost < self.outs[i][1]:
                self.outs.insert(i,(node_to_add, cost))
                added = True
                break
            
        if added == False:
            self.outs.append((node_to_add, cost))

    def add_in(self, node_to_add):
        """
        Adds a node to the node's list of ins.

        Args:
            node_to_add (Node): The node object to be added as an in.
        """
        if node_to_add not in self.ins:
            self.ins.append(node_to_add)

    def to_dict(self):
        """
        Returns a dictionary representation of the node's outs

        Returns:
            dict: The outs in dictionary form
        """
        return {
            "outs": [{"id": out_node[0].id, "cost": out_node[1]} for out_node in self.outs]
        }  

DEG_TO_RAD = np.pi / 180
R = 6378.137 #Earth radius in km
def calc_distance(lat1, lon1, lat2, lon2):
    """
    Calculates the distance between 2 sets of coordinates.

    Args:
        lat1 (float): The latitude coordinate of point 1
        lon1 (float): The longitude coordinate of point 1
        lat2 (float): The latitude coordinate of point 2
        lon2 (float): The longitude coordinate of point 2

    Returns:
        The distance between points in metres
    """

    dLat: float = (lat2 - lat1) * DEG_TO_RAD
    dLon: float = (lon2 - lon1) * DEG_TO_RAD

    a: float = np.sin(dLat / 2) ** 2 + np.cos(lat1 * DEG_TO_RAD) * np.cos(lat2 * DEG_TO_RAD) * np.sin(dLon / 2) ** 2

    c: float = 2 * np.arctan2(np.sqrt(a), np.sqrt(1 - a))
    return R * c * 1000  # metres