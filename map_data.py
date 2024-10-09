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

    for i in road_osm['elements']:
        if i['type'] == "way":
            nodes = i['nodes']

            #Looping all nodes in a given way
            for i in range(len(nodes)):
                #Initiated node in graph
                if (nodes[i] not in graph):
                    graph[nodes[i]] = node.Node(nodes[i],(0.0,0.0))

                #Adding nodes either side in way list to node's out list

                if i != 0:
                    graph[nodes[i]].add_out(graph[nodes[i-1]])

                if i != len(nodes)-1:
                    #Index ahead in array now, so nodes may not have been init here
                    if (nodes[i+1] not in graph):
                        graph[nodes[i+1]] = node.Node(nodes[i+1],(0.0,0.0))

                    graph[nodes[i]].add_out(graph[nodes[i+1]])
    return graph

if __name__=="__main__":
    graph = read_data()


    G = nx.Graph()

    G.add_nodes_from(graph)

    for node in graph:
        for edge in graph[node].outs:
            G.add_edge(graph[node].id,edge.id)

    options = {
        'node_color': 'red',
        'node_size' : 10,
    }

    nx.draw_networkx(G, **options)

    plt.savefig("graph.pdf")
    plt.show()