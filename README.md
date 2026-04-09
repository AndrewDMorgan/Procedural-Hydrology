# Procedural-Hydrology
Procedural hydrology, based not on erosion, but instead a novel technique.

I'm back from hibernation, and this time it's not yet another ISA (for now...).

The algerithm works first by having known continents. A distance field is then created from the shore.

The second step is to select random points a certain distance from the shore onto the land, and place them around at a given max density (a hash map binning system was used to prevent overlapping points).

From there, those points, using a breadth first search, propagate downhill based on the gradient, and an attraction force towards other rivers. When colliding the rivers combine strengths, but continue as seperate beings (to ensure proper behavior down the line)

After this, another pass is done that takes the points, finds the first and second derivative, and based on how perpendicular the acceleration force is to the velocity (calculated, not the same one used to propagate the rivers), the river is pushed towards the acceleration. This bends the rivers on corners, smoothing them, and exaggerating them for a better and smoother look.

From here, a distance map is created from all rivers, such that at a given point there is a distance map from the ocean, and a distance map of all rivers. This will allow merging of terrain without overlapping the rivers or oceans.

The rest of these steps aren't fully fleshed out and are all WIP.

A height map for all rivers. Nearby rivers need to share similar heights. The height of a river has to continuously remain the same, or increase when moving from the coast inland (or else it'd be going uphill).

Mainly, a height map is needed (doesn't matter from where, similar to the continents) and that is blended such that the coast flattens out (and tries to often generate flat regions or flood plains near the ocean, but also cliffs or canyons) and moves to the river's height when nearby (ensure to rise at least some from the river). This should prevent the river from flowing uphill, and prevent it from looking like it's floating or a massive trench as the terrain builds from the river. A custom system could also be built from the height map of the rivers and then used to create these features more explicity.


This system could be turned into a chunk based one if done correctly, such that it could be done chunk by chunk (likely neighboring chunks would need to be calculated to load a chunk, and then recalculated when they themself is loaded.

Performance isn't great, but also isn't terrible. This is also just a prototype right now.

Land Generation Took 551.30225ms      (not this land generation/continent formation is just a placeholder, and not at all final. It would be replaced with something much faster)
Distance Mask Generation (And River Locations) Took 69.971584ms
  3012 Starts     (this means it created 3012 rivers in total across a sample world of 750x750)
River Path Generation Took 189.668458ms
Generated Gradients And Bended Rivers In 2.609709ms
Generate Distances To Rivers In 87.80175ms
Total Elapsed Time: 901.527584ms

