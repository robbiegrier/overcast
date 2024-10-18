
## Todo

### search algorithm
- return path from one building to another using the roadmap

### road placement - detect sides
- attach road to existing roads along the length of the placed road

### basic ui
- buttons and text for keybinds to change modes and toggle visualizations

### vehicle ai
- spawn at random building
- calculate path to random destination building
- on road segments, straight line in desired direction
    - start with one lane and no collision
- on intersections, straight, turn, or uturn for desired new road segment
    - passing cars just go through each other
- on final segment, delete when adjacent to destination building

### speed limiter
- detect vehicle in front
- limit speed so that you don't collide with the vehicle in front
- vehicles have random speed offsets to test effect

### lane lines
- overlay roads with appropriate lane lines

### lane change
- change lane on large road based on next intersection turn direction
- interpolate left or right

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