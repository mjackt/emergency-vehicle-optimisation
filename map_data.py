import json
import node
import matplotlib.pyplot as plt
import networkx as nx


def read_data():
    road_osm_file = open('roadOSM.json')

    road_osm = json.load(road_osm_file)

    road_osm_file.close()

    #dictionary of Nodes
    graph = {}

    #Init all nodes. Could be done in loop with checking ways but this solves finding node locations to init them
    for i in road_osm['elements']:
        if i['type'] == "node":
            graph[i['id']] = node.Node(i['id'],(i['lat'],i['lon']))

    #Handling ways
    for i in road_osm['elements']:
        if i['type'] == "way":
            nodes = i['nodes']

            oneway=False

            if 'oneway' in i['tags']:
                oneway = True

            #Looping all nodes in a given way
            for i in range(len(nodes)):
                #Adding nodes either side in way list to node's out list

                if i != 0 and oneway == False:
                    graph[nodes[i]].add_out(graph[nodes[i-1]])

                if i != len(nodes)-1:
                    graph[nodes[i]].add_out(graph[nodes[i+1]])
    return graph

if __name__=="__main__":
    graph = read_data()


    G = nx.DiGraph()

    for node in graph:
        #Swapped solves orientation issues. Now weirdly stretched
        pos=graph[node].location[1], graph[node].location[0]
        G.add_node(node, pos=pos)

    for node in graph:
        for edge in graph[node].outs:
            G.add_edge(graph[node].id,edge.id)

    options = {
        'node_color': 'red',
        'node_size' : 10,
    }

    pos=nx.get_node_attributes(G,'pos')

    nx.draw_networkx(G, pos)

    plt.savefig("graph.pdf")
    plt.show()