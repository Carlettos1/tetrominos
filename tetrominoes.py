#!/usr/bin/env python3

import sys, os
from random import randint, random, choice, gammavariate
from math import *

pieces = dict()

pieces['C'] = [ [ (1, 1), (1, 0), (1, 1) ],    # %%
                                               # % 
                                               # %%

            [ (1, 0, 1), ( 1, 1, 1) ],         # % %
                                               # %%%

            [ (1, 1), (0, 1), (1, 1) ],        # %%
                                               #  %
                                               # %%

            [ (1, 1, 1), (1, 0, 1) ] ]         # %%%
                                               # % %

pieces['O'] =   [ [ (1, 1), (1, 1) ] ]         # %%
                                               # %%

pieces['J'] =    [ [ (1, 0), (1, 1) ],         # %
                                               # %%

                 [ (0, 1), (1, 1) ],           #  %
                                               # %%

                 [ (1, 1), (0, 1) ],           # %%
                                               #  %

                 [ (1, 1), (1, 0) ] ]          # %%
                                               # % 

pieces['I'] =   [ [ (1, 1, 1, 1), ],           # %%%%

                [ (1,), (1,), (1,), (1,) ] ]   # %
                                               # %
                                               # %
                                               # %

pieces['B'] =  [ [ (1, 1), ],                  # %%

                 [ (1,), (1,), ] ]             # %
                                               # %

pieces['T'] = [ [ (0, 1, 0), (1, 1, 1) ],      #  %
                                               # %%% 

            [ (0, 1), (1, 1), (0, 1) ],        #  % 
                                               # %%
                                               #  %

            [ (1, 1, 1), (0, 1, 0), ],         # %%%
                                               #  % 

            [ (1, 0), (1, 1), (1, 0) ] ]       # % 
                                               # %%
                                               # %

pieces['L'] = [ [ (1, 0), (1, 0), (1, 1), ],   # %
                                               # %
                                               # %%
                                               
            [ (0, 1), (0, 1), (1, 1) ],        #  %
                                               #  %
                                               # %%

            [ (0, 0, 1), (1, 1, 1), ],         #   %
                                               # %%%

            [ (1, 0, 0), (1, 1, 1), ],         # %  
                                               # %%%

            [ (1, 1), (0, 1), (0, 1), ],       # %%
                                               #  %
                                               #  %

            [ (1, 1), (1, 0), (1, 0), ],       # %%
                                               # %
                                               # %

            [ (1, 1, 1), (1, 0, 0) ],          # %%%
                                               # %  

            [ (1, 1, 1), (0, 0, 1) ] ]         # %%%
                                               #   %

pieces['S'] = [ [ (1, 0), (1, 1), (0, 1) ],    # %
                                               # %%
                                               #  %

           [ (0, 1, 1), (1, 1, 0) ],           #  %%
                                               # %%

           [ (0, 1), (1, 1), (1, 0) ],         #  % 
                                               # %%
                                               # % 

           [ (1, 1, 0), (0, 1, 1) ] ]          # %%
                                               #  %%


#
# The state of the board will be saved as a list of 4-tuples, each containing
# ( Shape, Orientation, X coordinate, Y coordinate )
# Where Shape is one of 'C', 'O', 'J', 'I', 'B', 'T', 'L', 'S'
# Orientation is an integer between 0 and len(pieces[Shape])-1
# X coordinate is an integer between 0 and 9
# Y coordinate is an integer between 0 and 14
#

#
# This state of the board can be "rendered" to a 10 x 15 matrix
# Where each cell is either empty, ocuppied by a shape, or invalid
# The cell is invalid if it is occupied by more than one shape
#
def Render(A):
    M = [ [ '#' for i in range(10) ] for j in range(15) ]
    for shape, orient, x, y in A:
        assert 0 <= orient < len(pieces[shape])
        for this_y, row in enumerate(pieces[shape][orient]):
            if not (0 <= y + this_y < 15): return None
            for this_x, w in enumerate(row):
                if not (0 <= x + this_x < 10): return None
                if M[y+this_y][x+this_x] != '#': M[y+this_y][x+this_x] = 'X'
                elif w == 1: M[y+this_y][x+this_x] = shape
    return M

def Visualize(M):
    print ('----------------------------------')
    for j in range(15):
        print ('|', end='')
        for i in range(10):
            print (' ', ' ' if M[j][i] == '#' else M[j][i], end='')
        print ('  |')
    print ('----------------------------------')
    print ()

#
# An energy can be defined for a "rendered" board,
# As the sum of all the (unique) pair distances,
# each distance raised to the power alpha, between empty cells
#

ALPHA = 3.0

def Energy(M):
    if M is None: return 1.0E+20
    EE = 0.0
    for j in range(15):
        for i in range(10):
            if M[j][i] == 'X': EE += 5.0E+5
            elif M[j][i] == '#':
               for n in range(15):
                   for m in range(10):
                       if m != i and n != j and M[n][m] == '#':
                          EE += ((m-i)**2 + (n-j)**2)**(0.5*ALPHA)
    return 0.5*EE

def ShapeCount(shape, state): return sum(1 if item[0] == shape else 0 for item in state)

E0 = Energy(Render([]))

mu = 25.0
var = 30.0
theta = var/mu

p_displace = 0.25
p_rotate = 0.25
p_add = 0.25
p_remove = 0.25

state = list()
Emin, step, best, best_step = None, 0, list(), None
init_date = os.popen('date').read().strip()
g = open('BEST.txt', 'w')
while True:
      if step % 500 == 0: state = [ w for w in best ]
      M, E = Render(state), None
      if step % 5000 == 0: 
         beta = gammavariate(mu/theta, theta)
         p_displace, p_rotate, p_add, p_remove = random(), random(), random(), random()
         ss = p_displace+p_rotate+p_add+p_remove
         p_displace, p_rotate, p_add, p_remove = p_displace/ss, p_rotate/ss, p_add/ss, p_remove/ss
      if M is not None:
         if step % 100 == 0: os.system('clear')
         E = Energy(M)
         if Emin is None or E < Emin:
            Emin = E
            best = [ w for w in state ]
            for w in best: print(w, file=g)
            print (file=g)
            g.flush()
            best_step = step
         if step % 100 == 0:
            print ('step ', step, ':    E/E0 = ', E/E0, '   beta = ', beta, ' probs = ', p_displace, p_rotate, p_add, p_remove)
            Visualize(M)
            print ()
            print ('Simulation started ', init_date)
            print ('E_best/E0 = ', Emin/E0, ' (best since step %d)' % best_step)
            print ('Available pieces: ', ' '.join(shape*(5-ShapeCount(shape, best)) for shape in pieces))
         M_best = Render(best)
         if step % 100 == 0: 
            print ('Empty cells: ', sum(1 if M_best[j][i] == '#' else 0 for j in range(15) for i in range(10) ))
            Visualize(M_best)
      if len(state) > 0 and random() < p_remove:
         # Proposal to remove a piece from the board
         state0 = [ w for w in state ]
         k = randint(0, len(state0)-1)
         state = [ w for i, w in enumerate(state) if i != k ] 
         newM = Render(state)
         newE = Energy(newM)
         if newE < E or log(random()) < -beta*(newE-E)/E0: pass
         else: state = state0
      if any((ShapeCount(shape, state) < 5) for shape in pieces) and random() < p_add:
         # Proposal to add a piece to the board
         state0 = [ w for w in state ]
         shapes = list()
         for s in pieces:
             if ShapeCount(s, state) < 5:
                for z in range(5-ShapeCount(s, state)): shapes.append(s)
         shape = choice(shapes)
         state.append((shape, randint(0, len(pieces[shape])-1), randint(0, 9), randint(0, 14)))
         newM = Render(state)
         newE = Energy(newM)
         if newE < E or log(random()) < -beta*(newE-E)/E0: pass
         else: state = state0
      if len(state) > 0 and random() < p_displace:
         # Proposal to displace a piece in the board
         state0 = [ w for w in state ]
         k = randint(0, len(state)-1)
         dx = randint(-1, 1)
         dy = randint(-1, 1)
         state[k] = tuple([ state[k][0], state[k][1], state[k][2]+dx, state[k][3]+dy ])
         newM = Render(state)
         newE = Energy(newM)
         if newE < E or log(random()) < -beta*(newE-E)/E0: pass
         else: state = state0
      if len(state) > 0 and random() < p_rotate:
         # Proposal to rotate a piece in the board
         state0 = [ w for w in state ]
         k = randint(0, len(state)-1)
         new_or = randint(0, len(pieces[state[k][0]])-1)
         state[k] = tuple([ state[k][0], new_or, state[k][2], state[k][3] ])
         newM = Render(state)
         newE = Energy(newM)
         if newE < E or log(random()) < -beta*(newE-E)/E0: pass
         else: state = state0
      step += 1
#
#
#
