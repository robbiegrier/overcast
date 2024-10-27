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