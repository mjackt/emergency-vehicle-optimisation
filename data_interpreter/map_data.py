import csv
import json
import os
import sys
import matplotlib.pyplot as plt
import networkx as nx
import random
import numpy as np
import json
import collections
import heapq
import jsonlines
import pandas as pd
import typing

from node import calc_distance
from node import Node


def read_data(place: str):
    """
    Reads required raw osm data (osm.json and police.json) from /input_data/*place*/

    Args:
        place (str): The name of the sub-folder in the input_data folder. Generally the name of the area covered by the osm data.

    Returns:
        dict{int: Node}: A graph representaion of the osm data (Unpruned).
    """
    #182576008...1254203137 - Plymouth broken one way trap
    #547154223...341619748 - Fowey Broken 1 ways. (Wrong order of nodes)
    #30620881 - Not broken themselves but reliant on Fowey broken bit so need removing
    BROKEN_IDS = [182576008, 1254204193, 1254204194, 785806394, 1018262965, 1254200712, 1254200713, 1254203133, 1254203134, 1254203136, 1254203137, 547154223, 29285959, 1061735928, 58947176, 167067602, 58947175, 164364892, 58947189, 341619748, 30620881]#List of known issues in OSM data that my program will ignore.

    road_osm_file = open('input_data/'+ place +'/osm.json')
    road_osm = json.load(road_osm_file)
    road_osm_file.close()

    graph: typing.Dict[int, Node] = {}

    try:
        data_months: int = len(next(os.walk('input_data/call_data'))[1])
    except:
        data_months: int = 0

    #Init all nodes. Could be done in loop with checking ways, but this solves finding node locations to init them
    for i in road_osm['elements']:
        if i['type'] == "node":
            inc: int = 0
            if data_months == 0:#If theres no data
                inc = 1

            graph[i['id']] = Node(i['id'],(i['lat'],i['lon']), int(inc))

    #OSM ways represnt stretches of roads. By going through them we can build edges into the inited nodes
    for i in road_osm['elements']:
        if i['type'] == "way":
            if i['id'] in BROKEN_IDS:
                continue

            nodes: list[int] = i['nodes']

            oneway: bool = False

            if 'oneway' in i['tags']:
                if i['tags']['oneway'] == 'yes' or i['tags']['oneway'] == "-1":
                    oneway = True

            speed: float = 30.0#Default speed of 30mph seems reasonable
            if 'maxspeed' in i['tags']:
                speed = int(i['tags']['maxspeed'][0:2])#Use real speed if data available. Most of the time it is available

            speed: float = speed / 2.237  #mph to m/s

            #Looping all nodes in a given way
            for i in range(len(nodes)):
                #Adding nodes either side in way list to node's out list
                if i != 0 and oneway == False:
                    graph[nodes[i]].add_out(graph[nodes[i-1]], speed)
                    graph[nodes[i-1]].add_in(graph[nodes[i]])

                if i != len(nodes)-1:
                    graph[nodes[i]].add_out(graph[nodes[i+1]], speed)
                    graph[nodes[i+1]].add_in(graph[nodes[i]])

    #Creating police nodes
    police_osm_file: typing.File = open('input_data/'+ place +'/police.json')

    police_osm: dict = json.load(police_osm_file)

    police_osm_file.close()

    police_ids: list[int] = []#Stores node id representing each police station
    police_names: list[str] = []#Stores names of each police station. index 0 is the name of index 0 in police_ids. Used for displaying results of GA

    for i in police_osm['elements']:
        #Some stations are represented by a way. We can just select any node in that way to represnt it.
        if i['type'] == "way":
            if 'tags' in i:
                police_names.append(i['tags']['name'])
                node_id: int = i['nodes'][0]#Selecting first node as representative
                
                for element in police_osm['elements']:
                    #Find representative's coords
                    if element['type'] == 'node' and element['id'] == node_id:
                        lat: float = element['lat']
                        long: float = element['lon']

                #Create new node obj for police station
                node_to_add: Node = Node(node_id, (lat, long), 0, True)
                #Find it's nearest road node
                nearest: Node = find_nearest_node(node_to_add, graph)

                #Add connections between the two nodes
                node_to_add.add_out(nearest, 30)
                node_to_add.add_in(nearest)
                nearest.add_out(node_to_add, 30)
                nearest.add_in(node_to_add)

                graph[node_id] = node_to_add
                police_ids.append(node_id)

        elif i['type'] == "node":
            if 'tags' in i:
                police_names.append(i['tags']['name'])
                #Create new node obj for police station
                node_to_add: Node = Node(i['id'], (i['lat'], i['lon']), 0, True)
                #Find it's nearest road node
                nearest: Node = find_nearest_node(node_to_add, graph)

                #Add connections between the two nodes
                node_to_add.add_out(nearest, 30)
                node_to_add.add_in(nearest)
                nearest.add_out(node_to_add, 30)
                nearest.add_in(node_to_add)
                
                graph[i['id']] = node_to_add
                police_ids.append(i['id'])

        #Finally some stations are represented by relations which are collections of ways. We can select first node from first way , similar to how ways are handled
        elif i['type'] == "relation":
            police_names.append(i['tags']['name'])
            way_id: int = i['members'][0]['ref']

            for element in police_osm['elements']:
                if element['type'] == 'way' and element['id'] == way_id:
                    node_id: int = element['nodes'][0]
                    break
            
            for element in police_osm['elements']:
                if element['type'] == 'node' and element['id'] == node_id:
                    lat: float = element['lat']
                    long: float = element['lon']
            #Find it's nearest road node
            node_to_add: Node = Node(node_id, (lat, long), 0, True)
            nearest: Node = find_nearest_node(node_to_add, graph)

            #Add connections between the two nodes
            node_to_add.add_out(nearest, 30)
            node_to_add.add_in(nearest)
            nearest.add_out(node_to_add, 30)
            nearest.add_in(node_to_add)

            graph[node_id] = node_to_add
            police_ids.append(node_id)
            
    with open('out/police_ids.json', 'w') as f:
        json.dump(police_ids, f)

    with open('out/police_names.json', 'w') as f:
        json.dump(police_names, f)

    return graph

def find_nearest_node(target, graph: dict):
    """
    Takes a node and returns the nearest other node to it

    Args:
        target (Node): The node to find nearest to

    Returns:
        Node: The nearest node to target.
    """
    nearest: Node = None
    nearest_dist: float = sys.maxsize

    for node in graph.values():
        dist: float = calc_distance(target.location[0], target.location[1], node.location[0], node.location[1])

        if dist < nearest_dist:
            nearest = node
            nearest_dist = dist

    return nearest

def remove_pits(graph: dict):    
    """
    Removes nodes in the graph that have 0 ins or 0 outs

    Args:
        graph (dict{int: Node}): The original graph of nodes

    Returns:
        dict{int: Node}: Updated graph
    """
    while True:
        to_remove: list[Node] = []
        for _, node in graph.items():
            if len(node.outs) == 0 or len(node.ins) == 0:
                to_remove.append(node)
    
        if len(to_remove) == 0:
            break
        else:
            for node in to_remove:
                graph = remove_pit(node, graph)

    return graph

def keep_decisions(graph: dict):
    """
    Removes nodes that only have one available out (without backtracking).

    Args:
        graph (dict{int: Node}): The original graph of nodes

    Returns:
        dict{int: Node}: Updated graph
    """
    while True:
        to_remove_2way: list[Node] = []
        to_remove_1out: list[Node] = []
        delete_list: list[Node] = []
        for _, node in graph.items():
            if len(node.outs) == 0:#Disconnected sections will reduce to one node
                delete_list.append(node)
            elif len(node.outs) > 2 or node.police:#Keep the nodes fulfilling this
                continue
            elif len(node.outs) == 2:#Just having two outs doesnt immediatley qualify for removal. We need to check its part of a 2 way road with 2 ins matching it's 2 outs
                out_ids: list[int] = []
                in_ids: list[int] = []

                for out in node.outs:
                    out_ids.append(out[0].id)

                for entry in node.ins:
                    in_ids.append(entry.id)

                #See if in ins and outs match (i.e normal 2 way road with no intersection)
                if collections.Counter(in_ids) == collections.Counter(out_ids) and node.outs[0][0] != node.outs[1][0]:
                    to_remove_2way.append(node)
            else:#1 out. No checks needed. Just foward all ins to it's one out
                if node.outs[0][0] not in node.ins:#Prevent pointing to self
                    to_remove_1out.append(node)


        if len(delete_list) == 0 and len(to_remove_2way) == 0 and len(to_remove_1out) == 0:
            break
        else:
            for node in delete_list:
                for entry in node.ins:
                    index_to_go: int = None
                    for i in range(len(entry.outs)):
                        if entry.outs[i][0].id == node.id:
                            index_to_go = i
                            break
                    entry.outs.pop(index_to_go)

                del graph[node.id]
                
            for node in to_remove_2way:
                graph = prune_2way_node(graph, node)
            
            for node in to_remove_1out:
                graph = prune_1out_node(graph, node)

    return graph

def prune_1out_node(graph: dict, node):
    """
    Removes a non-decision node with a single out

    Args:
        graph (dict{int: Node}): The original graph of nodes
        node (Node): The node to remove

    Returns:
        dict{int: Node}: Updated graph
    """
    #Pass on incident probabiltiy to nearest neighbour
    node.outs[0][0].incid_in_year += node.incid_in_year
    exit: Node = node.outs[0][0]

    if exit in node.ins:#Checking if this exit immediatley returns to the node to remove.#
                        #Sometimes there will be one way circles i.e car parks which if unchecked results in node pointing to itself which causes issues
        return graph

    new_ins: list[Node] = []
    #New in list for the exit node, excluding the node being removed
    for into in exit.ins:
        if into.id != node.id:
            new_ins.append(into)

    #Now we need to point all nodes going into prunee towards the only out of prunee
    for entry in node.ins:#Iterate over everying coming into prunee
        #Go over the entry's outs and find the index of the prunee
        for i in range(len(entry.outs)):
            if entry.outs[i][0].id == node.id:
                index_of_pruned: int = i
                break
        if entry in exit.ins:#Check if the entry already points to prunee's exit
            graph[entry.id].outs.pop(index_of_pruned)
            continue
        
        #Current distance from prunee -> exit
        dist_to_exit: float = node.outs[0][1]
        
        combined_dist: float = entry.outs[index_of_pruned][1] + dist_to_exit#Add on dist from entry to prunee
        new_out: tuple[Node, float] = (exit, combined_dist)#Create new out tuple representing entry -> exit (bypassing prunee) with the calculated combined cost
        
        graph[entry.id].outs[index_of_pruned] = new_out#Replace the out previously pointing to prunee with newly created one

        new_ins.append(entry)#Add entry as a new in for exit node

    graph[exit.id].ins = new_ins#Provide exit with it's update ins list
    graph.pop(node.id)#Finally remove prunee from graph
    return graph

def prune_2way_node(graph: dict, node):
    """
    Removes a non-decision node that must be part of a 2 way road             
    Args:
        graph (dict{int: Node}): The original graph of nodes
        node (Node): The node to remove

    Returns:
        dict{int: Node}: Updated graph
    """
    if len(node.outs) != 2 or len(node.ins) != 2:#Pruning may have circled back round turning a 2 way node into a leaf. So this just makes that not an issue and it will prob be removed by leaf pruning
        return graph
    #Pass on incident probabiltiy to nearest neighbour
    node.outs[0][0].incid_in_year += node.incid_in_year

    prevent_double_point: bool = False
    if node.ins[0] in node.ins[1].ins:
        prevent_double_point = True

    for entry in node.ins:#We need to iterate over both entries into the prunee and point each one to the other entry (the only possible exit).
        if node.ins.index(entry) == 0:
            other_node: Node = node.ins[1]
        else:
            other_node: Node = node.ins[0]

        #Figuring out which out needs to be redirected
        for i in range(len(entry.outs)):
            if entry.outs[i][0].id == node.id:
                index_of_pruned = i
                break
        

        if prevent_double_point:#If the other 2 nodes are already connected then we can just delete the prunee
            graph[entry.id].outs.pop(index_of_pruned)
            graph[entry.id].ins.remove(node)
            continue

        for out in node.outs:
            if out[0].id == other_node.id:
                dist_to_other_node: float = out[1]#Distance from prunee -> to the exit whic is not the current entry
        
        combined_dist: float = entry.outs[index_of_pruned][1] + dist_to_other_node#Add distance from entry -> prunee
        new_out: tuple[Node, float] = (other_node, combined_dist)#Create new tuple to represent out going from entry -> other_node

        graph[entry.id].outs[index_of_pruned] = new_out#Update out previously pointing to prunee to now point to other_node

        #Create new list of ins for the other_node by excluding prunee
        new_ins: list[Node] = []
        for into in other_node.ins:
            if into.id != node.id:
                new_ins.append(into)
        #Add the entry to other_node's new list of ins
        new_ins.append(entry)
        graph[other_node.id].ins = new_ins#Update in graph
    
    graph.pop(node.id)#Finally remove prunee from graph

    return graph

def remove_tiny_leaves(graph: dict, agg_limit: float):
    """
    Checks graph for leafs under the agglomerate limit and removes them            
    Args:
        graph (dict{int: Node}): The original graph of nodes
        agg_limit (float): Agglomerate limit. Leafs under this distance from their nearest node will be deleted

    Returns:
        dict{int: Node}: Updated graph
    """
    tiny_leaves: list[Node] = []
    for node in graph.values():
        if node.police == False and len(node.outs) == 1 and node.outs[0][0].id == node.ins[0].id and len(node.ins) == 1:#Checks if node is a leaf
            if node.outs[0][1] < agg_limit:
                tiny_leaves.append(node)

    for leaf in tiny_leaves:
        remove_leaf(leaf, graph)

    return graph

def remove_leaf(node, graph: dict):
    """
    Remove a leaf from the graph          
    Args:
        graph (dict{int: Node}): The original graph of nodes
        node (Node): The node to be deleted

    Returns:
        dict{int: Node}: Updated graph
    """
    if node.police == False:
        graph.pop(node.id)
        if len(node.outs) > 0:#Disconnected sections will sometimes end up being just two points. Means when ones removed there are no outs
            exit: Node = node.outs[0][0]
            for i in range(len(exit.ins)):
                if exit.ins[i].id == node.id:
                    index_to_remove: int = i
                    break

            exit.ins.pop(index_to_remove)

            for i in range(len(exit.outs)):
                if exit.outs[i][0].id == node.id:
                    index_to_remove: int = i
                    break

            exit.outs.pop(index_to_remove)

    return graph

def prune_graph(graph: dict, agg_limit: float):
    """
    Prunes the graph.
    Removes pits, non-decisions and small leafs       
    Args:
        graph (dict{int: Node}): The original graph of nodes
        agg_limit (float): Agglomerate limit. Leafs under this distance from their nearest node will be deleted

    Returns:
        dict{int: Node}: Updated graph
    """
    graph: dict = remove_pits(graph)
    changes: bool = True
    while changes == True:
        start_len: int = len(graph)
        graph: dict = keep_decisions(graph)
        graph: dict = remove_tiny_leaves(graph, agg_limit)
        if start_len == len(graph):
            changes: bool = False
    return graph

def remove_pit(end, graph: dict):
    """
    Removes a node that is a pit        
    Args:
        graph (dict{int: Node}): The original graph of nodes
        end (Node): Node to be removed

    Returns:
        dict{int: Node}: Updated graph
    """
    if end.police == False:
        graph.pop(end.id)

        for entry in end.ins:
            new_outs: tuple[Node, float] = []
            for out in entry.outs:
                if out[0].id != end.id:
                    new_outs.append(out)

            entry.outs = new_outs

        for out in end.outs:
            new_ins: list[Node] = []
            for entry in out[0].ins:
                if entry.id != end.id:
                    new_ins.append(entry)

            out[0].ins = new_ins

    return graph

def dijkstra(start_node, current_apsp):
    """
    REDUNDANT
    """
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
    """
    REDUNDANT
    """
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
    """
    REDUNDANT
    """
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
    """
    Checks that every node's outs actually exist      
    Args:
        graph (dict{int: Node}): The original graph of nodes
    """
    for node in graph.values():
        for out in node.outs:
            if out[0].id not in graph:
                print(str(node.id) + " has non existent out: " + str(out[0].id))

def truncate(f, n):
    """
    Truncates/pads a float f to n decimal places without rounding

    Args:
        f (float): Float to truncate
        n (int): Number of decimal places to truncate to

    Returns:
        float: The truncated float
    """
    s = '{}'.format(f)
    if 'e' in s or 'E' in s:
        return '{0:.{1}f}'.format(f, n)
    i, p, d = s.partition('.')
    return '.'.join([i, (d+'0'*n)[:n]])

if __name__=="__main__":
    graph: dict = read_data('dnc')

    AGGLOMERATE_LIMIT = 50.0

    print("Data read")
    print("Graph size: " + str(len(graph)))

    graph = prune_graph(graph, AGGLOMERATE_LIMIT)

    print("Graph pruned to " + str(len(graph)) + " nodes")

    data_months = len(next(os.walk('input_data/call_data'))[1])  
    counter = 0
    for root,_,files in os.walk("input_data/call_data"):
        distance_map: dict = {}#Gotta refresh every new file otherwise it gets to big and crashes
        for file in files:
            print(str(counter) + "/" + str(data_months))
            counter += 1
            if file.endswith(".csv"):
                with open(str(root) + "/" + str(file), 'r') as file:
                    reader = csv.DictReader(file)
                    i=0
                    for row in reader:
                        if i == 6000:
                            distance_map = {}#Prevent memory crash

                        print(str(counter) + "." + str(i))
                        i += 1
                        if row['Latitude'] == '':
                            continue
                        nearest = None
                        nearest_dist = sys.maxsize
                        for node in graph.values():
                            inc_lat_3dp = row['Latitude'][:6]
                            inc_lon_3dp = row['Longitude'][:6]
                            node_lat_3dp = truncate(node.location[0], 3)
                            node_lon_3dp = truncate(node.location[1], 3)
                            if (node_lat_3dp, node_lon_3dp, inc_lat_3dp, inc_lon_3dp) not in distance_map:                                
                                dist = calc_distance(float(row['Latitude']), float(row['Longitude']), node.location[0], node.location[1])
                                distance_map[(node_lat_3dp, node_lon_3dp, inc_lat_3dp, inc_lon_3dp)] = dist
                            else:
                                dist = distance_map[(node_lat_3dp, node_lon_3dp, inc_lat_3dp, inc_lon_3dp)]

                            if dist < 350:#If the distance is less than 300 metres then just stop the search cause this is good enough
                                nearest = node
                                nearest_dist = dist
                                break
                            elif dist < nearest_dist:
                                nearest = node
                                nearest_dist = dist

                        nearest.incid_in_year += (1/data_months*12)#Converting to a per year value

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
        pos = graph[node].location[1], graph[node].location[0]
        color = 'r' if graph[node].police else 'b'  # Red for police, blue otherwise
        G.add_node(node, pos=pos, color=color)

    for node in graph:
        for edge in graph[node].outs:
            G.add_edge(graph[node].id, edge[0].id, weight=edge[1])

    # Get positions and colors
    pos = nx.get_node_attributes(G, 'pos')
    colors = [G.nodes[node]['color'] for node in G.nodes()]

    # Draw nodes with specified colors
    nx.draw_networkx(G, pos, with_labels = False, node_size = 10, node_color = colors)

    #plt.savefig("out/graph.pdf")
    #plt.show()