class Node:
#List of out nodes
    def __init__(self, id, location):
        #id provided by OSM
        self.id: int = id
        #All nodes reachable from current node. != to all nodes that can reach this node due to one ways
        self.outs: list[Node] = []
        #Coords of node
        self.location: tuple[float,float] = location

    def __str__(self):
        return f"{self.id}\n{self.location}\n{self.outs}\n"
    
    def __repr__(self):
        out_list = []
        for node in self.outs:
            out_list.append(node.id)

        return f"\n {self.id}\n{self.location}\n{out_list}\n\n"

    def add_out(self, node_to_add):
        self.outs.append(node_to_add)