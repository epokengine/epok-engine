# Recreating a project for the Actor architecture

The Actor architecture is a breaking change. Previous Entity/Behaviour maps, older Blueprint versions and `.epokscript` sidecars are rejected with a diagnostic. Opening them does not rewrite or convert their files.

Create a new project using the current Basic, Sample or Third Person template. Copy reusable source assets into the new project, recreate the map with Actors, and reimplement scripts as C++ or Blueprint ActorComponents or Actor subclasses. Keep the original project separately while recreating its content.

Maps now store version 6 with one `actors` collection. Spatial values are stored on root components, custom logic is attached as ActorComponents, and C++ metadata comes from annotated classes. Blueprints use version 5 and the generated reflection cache uses schema 9.

There is no automatic compatibility or migration layer. See [Actors and components](actors.md) for the creation, attachment and reuse workflow.
