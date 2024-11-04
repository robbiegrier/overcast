
## Todo

### road placement - detect sides
- attach road to existing roads along the length of the placed road

### lane lines
- overlay roads with appropriate lane lines

### update vehicle paths on road change
- vehicles observe roads and detect changes
- recalculate paths if the roads change

### intersection rules
- rotate with simple timer
- start with single incoming direction
- vehicle ai can inspect state of interesection and stop or go

## Improvements

### road joints
- attach roads using a joint system
- bfs for possible joint in radius
- joint for split perpendicular, intersection, or extension
- road can be created clicking anywhere within the radius of the joint
- closest joint automatically selected when conflict

### building drag spawn
- click and drag to spawn rectangular building in dragged area

### skybox
- add skybox to make it look nicer

### better pathfinding
- use a better algorithm to get a realistic path