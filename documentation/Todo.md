
## Todo

### initial game state
- spawn some roads and buildings on startup

### lane change
- change lane on large road based on next intersection turn direction
- interpolate left or right

### road placement - detect sides
- attach road to existing roads along the length of the placed road

### basic ui
- buttons and text for keybinds to change modes and toggle visualizations

### lane lines
- overlay roads with appropriate lane lines

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