import json
import node
import matplotlib.pyplot as plt
import networkx as nx
import random
import numpy as np
import json
import collections
import heapq


def read_data():
    road_osm_file = open('input_data/roadOSM.json')

    road_osm = json.load(road_osm_file)

    road_osm_file.close()

    #dictionary of Nodes
    graph = {}

    #Init all nodes. Could be done in loop with checking ways but this solves finding node locations to init them
    for i in road_osm['elements']:
        if i['type'] == "node":
            rand = np.random.poisson(0.1 , 1)[0]

            graph[i['id']] = node.Node(i['id'],(i['lat'],i['lon']), int(rand))

    #Handling ways
    for i in road_osm['elements']:
        if i['type'] == "way":
            nodes = i['nodes']

            oneway=False

            if 'oneway' in i['tags']:
                oneway = True

            speed: int = 30

            if 'maxspeed' in i['tags']:
                speed = int(i['tags']['maxspeed'][0:2])

            speed = speed / 2.237  #mph to m/s


            #Looping all nodes in a given way
            for i in range(len(nodes)):
                #Adding nodes either side in way list to node's out list

                if i != 0 and oneway == False:
                    graph[nodes[i]].add_out(graph[nodes[i-1]], speed)
                    graph[nodes[i-1]].ins.append(graph[nodes[i]])

                if i != len(nodes)-1:
                    graph[nodes[i]].add_out(graph[nodes[i+1]], speed)
                    graph[nodes[i+1]].ins.append(graph[nodes[i]])
    return graph

def remove_dead_ends(graph: dict):    
    while True:
        to_remove = []
        for _, node in graph.items():
            if len(node.outs) == 0 or len(node.ins) == 0:
                to_remove.append(node)
    
        if len(to_remove) == 0:
            break
        else:
            for node in to_remove:
                graph = remove_node(node, graph)

    return graph

def keep_decisions(graph: dict):
    while True:
        to_remove = []
        for _, node in graph.items():
            if len(node.outs) != 2:
                continue
            else:
                out_ids = []
                in_ids = []

                for out in node.outs:
                    out_ids.append(out[0].id)

                for entry in node.ins:
                    in_ids.append(entry.id)

                if collections.Counter(in_ids) == collections.Counter(out_ids):
                    to_remove.append(node)

        if len(to_remove) == 0:
            break
        else:
            for node in to_remove:
                graph = prune_node(graph, node)

    return graph

def prune_node(graph: dict, node):
    node.outs[0][0].incid_in_year += node.incid_in_year
    for entry in node.ins:
        if node.ins.index(entry) == 0:
            other_node = node.ins[1]
        else:
            other_node = node.ins[0]

        for i in range(len(entry.outs)):
            if entry.outs[i][0].id == node.id:
                index_of_pruned = i
                break

        for out in node.outs:
            if out[0].id == other_node.id:
                dist_to_other_node = out[1]
        
        combined_dist = entry.outs[index_of_pruned][1] + dist_to_other_node
        new_out = (other_node, combined_dist)

        graph[entry.id].outs[index_of_pruned] = new_out

        new_ins = []
        for into in other_node.ins:
            if into.id != node.id:
                new_ins.append(into)

        new_ins.append(entry)
        graph[other_node.id].ins = new_ins
    
    graph.pop(node.id)

    return graph

def prune_graph(graph: dict):
    graph = remove_dead_ends(graph)
    graph = keep_decisions(graph)
    return graph

def remove_node(end, graph: dict):
    graph.pop(end.id)

    for entry in end.ins:
        new_outs = []
        for out in entry.outs:
            if out[0].id != end.id:
                new_outs.append(out)

        entry.outs = new_outs

    for out in end.outs:
        new_ins = []
        for entry in out[0].ins:
            if entry.id != end.id:
                new_ins.append(entry)

        out[0].ins = new_ins

    return graph

def dijkstra(start_node):
    # Dictionary to store the minimum cost to each node
    min_costs = {start_node.id: 0}
    # Dictionary to store the shortest path tree (predecessors)
    predecessors = {start_node.id: None}
    # Priority queue to select the node with the smallest cost
    priority_queue = [(0, start_node)]

    while priority_queue:
        # Pop the node with the smallest cost
        current_cost, current_node = heapq.heappop(priority_queue)

        # If the current cost is higher than the stored cost, skip processing
        if current_cost > min_costs[current_node.id]:
            continue

        # Iterate through the neighboring nodes
        for neighbor, cost in current_node.outs:
            # Calculate the total cost to reach the neighbor
            total_cost = current_cost + cost

            # If the calculated cost is lower than the stored cost, update
            if neighbor.id not in min_costs or total_cost < min_costs[neighbor.id]:
                min_costs[neighbor.id] = total_cost
                predecessors[neighbor.id] = current_node
                heapq.heappush(priority_queue, (total_cost, neighbor))

    return min_costs

if __name__=="__main__":
    graph: dict = read_data()

    graph = prune_graph(graph)

    probability_dict: dict = {}


    apsp = {}
    for node in graph.values():
        node_costs = dijkstra(node)
        apsp[node.id] = node_costs
    print("Generated apsp")

    with open('out/apsp.json', 'w') as f:
        json.dump(apsp, f)

    for id, obj in graph.items():
        probability_dict[id] = obj.incid_in_year

    with open('out/probs.json', 'w') as f:
        json.dump(probability_dict, f)


    G = nx.DiGraph()

    for node in graph:
        pos=graph[node].location[1], graph[node].location[0]
        G.add_node(node, pos=pos)

    for node in graph:
        for edge in graph[node].outs:
            G.add_edge(graph[node].id, edge[0].id, weight = edge[1])

    pos=nx.get_node_attributes(G,'pos')


    start = random.choice(list(graph.keys()))
    end = random.choice(list(graph.keys()))

    path = nx.dijkstra_path(G, start, end)

    path_edges = list(zip(path,path[1:]))
    nx.draw_networkx_nodes(G,pos,nodelist=path,node_color='r')
    nx.draw_networkx_edges(G,pos,edgelist=path_edges,edge_color='r',width=1)

    nx.draw_networkx(G, pos, with_labels = False, node_size = 10)

    plt.savefig("out/graph.pdf")
    plt.show()