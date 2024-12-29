import json
import sys
import matplotlib.pyplot as plt
import networkx as nx
import random
import numpy as np
import json
import collections
import heapq
import jsonlines

from node import calc_distance
from node import Node


def read_data():
    road_osm_file = open('input_data/osm.json')

    road_osm = json.load(road_osm_file)

    road_osm_file.close()

    #dictionary of Nodes
    graph = {}

    #Init all nodes. Could be done in loop with checking ways but this solves finding node locations to init them
    for i in road_osm['elements']:
        if i['type'] == "node":
            rand = np.random.poisson(0.1 , 1)[0]

            graph[i['id']] = Node(i['id'],(i['lat'],i['lon']), int(rand))

    #Handling ways
    for i in road_osm['elements']:
        if i['type'] == "way":
            nodes = i['nodes']

            oneway=False

            if 'oneway' in i['tags']:
                if i['tags']['oneway'] == 'yes':
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

    #Creating police nodes
    police_osm_file = open('input_data/police.json')

    police_osm = json.load(police_osm_file)

    police_osm_file.close()

    police_ids = []

    for i in police_osm['elements']:
        #Need to get one node to represent the way
        if i['type'] == "way":
            if 'tags' in i:
                node_id = i['nodes'][0]
                
                for element in police_osm['elements']:
                    if element['type'] == 'node' and element['id'] == node_id:
                        lat = element['lat']
                        long = element['lon']

                node_to_add = Node(node_id, (lat, long), 0, True)
                nearest = find_nearest_node(node_to_add, graph)

                node_to_add.add_out(nearest, 30)
                node_to_add.ins.append(nearest)

                nearest.add_out(node_to_add, 30)
                nearest.ins.append(node_to_add)

                graph[node_id] = node_to_add
                police_ids.append(node_id)

        elif i['type'] == "node":
            if 'tags' in i:
                node_to_add = Node(i['id'], (i['lat'], i['lon']), 0, True)
                nearest = find_nearest_node(node_to_add, graph)

                node_to_add.add_out(nearest, 30)
                node_to_add.ins.append(nearest)

                nearest.add_out(node_to_add, 30)
                nearest.ins.append(node_to_add)

                graph[i['id']] = node_to_add
                police_ids.append(i['id'])

        elif i['type'] == "relation":
            way_id = i['members'][0]

            for element in police_osm['elements']:
                if element['type'] == 'way' and element['id'] == way_id:
                    node_id = element['nodes'][0]
                    break
            
            for element in police_osm['elements']:
                if element['type'] == 'node' and element['id'] == node_id:
                    lat = element['lat']
                    long = element['lon']

            node_to_add = Node(node_id, (lat, long), 0, True)
            nearest = find_nearest_node(node_to_add, graph)

            node_to_add.add_out(nearest, 30)
            node_to_add.ins.append(nearest)

            nearest.add_out(node_to_add, 30)
            nearest.ins.append(node_to_add)

            graph[node_id] = node_to_add
            police_ids.append(node_id)
            
    with open('out/police.json', 'w') as f:
        json.dump(police_ids, f)

    return graph

def find_nearest_node(target, graph: dict):
    nearest = None
    nearest_dist = sys.maxsize

    for node in graph.values():
        dist = calc_distance(target.location[0], target.location[1], node.location[0], node.location[1])

        if dist < nearest_dist:
            nearest = node
            nearest_dist = dist

    return nearest

def remove_pits(graph: dict):    
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
        to_remove_2way = []
        to_remove_1out = []
        for _, node in graph.items():
            if len(node.outs) > 2 or node.police:
                continue
            elif len(node.outs) == 2:
                out_ids = []
                in_ids = []

                for out in node.outs:
                    out_ids.append(out[0].id)

                for entry in node.ins:
                    in_ids.append(entry.id)

                if collections.Counter(in_ids) == collections.Counter(out_ids):
                    to_remove_2way.append(node)
            else:#1 out
                if node.outs[0][0] not in node.ins:#Prevent pointing to self
                    to_remove_1out.append(node)


        if len(to_remove_2way) == 0 and len(to_remove_1out) == 0:
            break
        else:
            for node in to_remove_2way:
                graph = prune_2way_node(graph, node)
            
            for node in to_remove_1out:
                graph = prune_1out_node(graph, node)

    return graph

def prune_1out_node(graph: dict, node):
    #Pass on incident probabiltiy to nearest neighbour
    node.outs[0][0].incid_in_year += node.incid_in_year
    exit = node.outs[0][0]

    if exit in node.ins:#Sometimes there will be one way circles i.e car parks which if unchecked results in node pointing to itself which causes issues
        return graph
    
    new_ins = []
    for into in exit.ins:
        if into.id != node.id:
            new_ins.append(into)

    for entry in node.ins:
        for i in range(len(entry.outs)):
            if entry.outs[i][0].id == node.id:
                index_of_pruned = i
                break

        dist_to_exit = node.outs[0][1]
        
        combined_dist = entry.outs[index_of_pruned][1] + dist_to_exit
        new_out = (exit, combined_dist)

        graph[entry.id].outs[index_of_pruned] = new_out

        new_ins.append(entry)
    
    graph[exit.id].ins = new_ins
    
    graph.pop(node.id)

    return graph


def prune_2way_node(graph: dict, node):
    #Pass on incident probabiltiy to nearest neighbour
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

def remove_tiny_leaves(graph: dict, agg_limit: float):
    tiny_leaves = []
    for node in graph.values():
        if node.police == False and len(node.outs) == 1 and node.outs[0][0].id == node.ins[0].id and len(node.ins) == 1:#Checks if node is a leaf
            if node.outs[0][1] < agg_limit:
                tiny_leaves.append(node)

    for leaf in tiny_leaves:
        remove_leaf(leaf, graph)

    return graph

def remove_leaf(node, graph: dict):
    if node.police == False:
        graph.pop(node.id)
        if len(node.outs) > 0:#Disconnected sections will sometimes end up being just two points. Means when ones removed there are no outs
            exit = node.outs[0][0]
            for i in range(len(exit.ins)):
                if exit.ins[i].id == node.id:
                    index_to_remove = i
                    break

            exit.ins.pop(index_to_remove)

            for i in range(len(exit.outs)):
                if exit.outs[i][0].id == node.id:
                    index_to_remove = i
                    break

            exit.outs.pop(index_to_remove)

    return graph


def prune_graph(graph: dict, agg_limit: float):
    graph = remove_pits(graph)
    graph = keep_decisions(graph)
    graph = remove_tiny_leaves(graph, agg_limit)
    return graph

def remove_node(end, graph: dict):
    if end.police == False:
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

def dijkstra(start_node, current_apsp):
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

        #if current_node.id in apsp:
            #We have a load of information already
            #Add already known cost IF current_node is closer to target than start_node
            #ONCE HERE ADD EVERY NOT YET FOUND NODE INTO MIN COSTS WITH COST TO HERE PLUS APSP VALUE AND END

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

def first_stage(graph):
    node_count = len(graph)
    apsp = {}

    for node in graph.keys():
        apsp[node] = {}

    explore_q = [(random.choice(list(graph.values())), 0)]
    explored = []
    current_node = None
    exit = False
    #Will break if there are unreachable sections
    while len(explored) < node_count:
        while current_node in explored or current_node == None:
            try:
                current_node = explore_q.pop(0)[0]
            except IndexError as e:
                print("Discovered " + str(len(explored))+ "/" + str(node_count))
                return apsp, explored



        apsp[current_node.id][current_node.id] = 0.0

        ordered_outs = []
        for out in current_node.outs:
            apsp[current_node.id][out[0].id] = out[1]

            if len(ordered_outs) == 0:
                ordered_outs.append(out)

            else:
                added = False
                for i in range(len(ordered_outs)):
                    if out[1] < ordered_outs[i][1]:
                        ordered_outs.insert(i, out)
                        added = True
                        break
                    
                if added == False:
                    ordered_outs.append(out)

        for neighbour in ordered_outs:
            known_costs_from_neighbour = apsp[neighbour[0].id]

            for x in known_costs_from_neighbour.keys():
                est_cost = apsp[current_node.id][neighbour[0].id] + apsp[neighbour[0].id][x]

                if x not in apsp[current_node.id]:
                    apsp[current_node.id][x] = est_cost               
                elif est_cost < apsp[current_node.id][x]:
                    apsp[current_node.id][x] = est_cost

        explored.insert(0, current_node)
        
        #Insertion into explore q
        for neighbour in ordered_outs:
            if len(explore_q) == 0:
                explore_q.append(neighbour)

            else:
                added = False
                for i in range(len(explore_q)):
                    if neighbour[1] < explore_q[i][1]:
                        explore_q.insert(i, neighbour)
                        added = True
                        break
                if added == False:
                    explore_q.append(neighbour)

    return apsp, explored

def trenchard(graph):
    apsp, explored = first_stage(graph)

    explored_nodes = len(explored)
    print(explored_nodes)

    print("First step complete")

    #Reverse fill
    node_count = len(graph)
    j = 0

    targets = []
    for i in range(len(explored)+1):
        targets.append(i)
    while len(explored) != 0:
        node = explored[j]
        knowledge = apsp[node.id]


        if len(knowledge) >= explored_nodes:
            explored.remove(explored[j])
            print("Filled " + str(node_count - len(explored)))
            j = j-1

        else:
            #knowledge = {k: v for k, v in sorted(knowledge.items(), key=lambda item: item[1])}
            neighbours = sorted(node.outs, key=lambda x: x[1])

            for i in range(len(neighbours)):
                answerer = neighbours[i][0].id

                other_knowledge = apsp[answerer]

                for key, value in other_knowledge.items():
                    if key not in knowledge:
                        apsp[node.id][key] = value + apsp[node.id][answerer]

            if len(targets) > 0:
                try:
                    while len(apsp[node.id]) >= targets[0]:
                        print(str(targets.pop(0))+ "/" + str(explored_nodes))
                        print(len(targets))
                except:
                    print("Filled")

        if j >= len(explored) - 1:
            j = 0
        else:
            j += 1

    return apsp

def verify_graph(graph: dict):
    for node in graph.values():
        for out in node.outs:
            if out[0].id not in graph:
                print("Missing " + str(out[0].id))

if __name__=="__main__":
    graph: dict = read_data()

    AGGLOMERATE_LIMIT = 45.0

    print("Data read")
    print("Graph size: " + str(len(graph)))

    graph = prune_graph(graph, AGGLOMERATE_LIMIT)

    print("Graph pruned to " + str(len(graph)) + " nodes")

    verify_graph(graph)

    probability_dict: dict = {}

    #counter = 0
    #d_count = 0
    #apsp = {}
    #detatchment = {}
    #for node in graph.values():
        #added = False
        #for out in node.outs:
            #if out[1] > AGGLOMERATE_LIMIT:#Neighbour too far away so we will do calculations
                #break#All outs are sorted so we can break knowing next ones are even further away

            #elif out[0].id in apsp:#We already have calulated neighbours paths so we will reuse it
                #total_detatchment = out[1] + detatchment[out[0].id]

                #if total_detatchment > AGGLOMERATE_LIMIT:#If neighbour has recieved it's paths from another node we need to check how far that node is from current node
                    #continue
                
                #else:
                    #added = True
                    #apsp[node.id] = apsp[out[0].id]#Reuse paths
                    #detatchment[node.id] = total_detatchment

                    #Add costs to neighbours manually because we have that info and otherwise costs to stolen node will be 0.
                    #for out in node.outs:
                    #    apsp[node.id][out[0].id] = out[1]

                    #break

        #if added == False:
         #   node_costs = dijkstra(node, apsp)
          #  apsp[node.id] = node_costs
           # detatchment[node.id] = 0.0#Original calculations so theres no detatchment
            #d_count += 1

        #counter += 1
        #if counter % 1000 == 0:
         #   print(counter)

    #apsp = trenchard(graph)
    #print("Generated apsp with only " + str(d_count) + " dijkstra calculations")

    #print("Writing apsp.....")
    #with jsonlines.open('out/apsp.json', 'w') as f:
    #    for key, value in apsp.items():
    #        f.write({key:value})
    #print("Completed")

    # Convert each Node object to a dictionary and save it in a list
    nodes_data = {node_id: node.to_dict() for node_id, node in graph.items()}
    
    with open('out/graph.json', 'w') as json_file:
        json.dump(nodes_data, json_file, indent=4)

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