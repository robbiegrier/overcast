## done

### simplify tool spawning methods
- Use another pattern for calling functions instead of passing all of the queries and refs every time
- Maybe use an event writer or create a new system that responds to all spawn events

### road extensions
- extend road segment (to and from) when placing new road.
- replace exisiting segment with longer segment

### roadmap graph structure
- model connections between intersections and road segments
- weighted edges based on segment length
- update on road segment edits

### buildings attached to roads
- segments in graph connected to adjacent buildings
- update on building and road edits

### search algorithm
- return path from one building to another using the roadmap

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

### initial game state
- spawn some roads and buildings on startup

### lane change
- change lane on large road based on next intersection turn direction
- interpolate left or right

### basic ui
- buttons and text for keybinds to change modes and toggle visualizations

### lane lines
- overlay roads with appropriate lane lines

### update vehicle paths on road change
- vehicles observe roads and detect changes
- delete vehicles on change