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
