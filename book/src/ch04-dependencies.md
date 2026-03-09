# Dependency Graphs and Execution Order

When a simulation contains multiple tasks, the order in which they execute **matters**. A controller must read sensor data before computing its output, and actuators must receive commands before driving hardware. This chapter covers how r-sim models and resolves these ordering constraints.
