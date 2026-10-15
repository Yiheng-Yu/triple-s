Creates a 'deep copy' of the current instance.
Unlike the `.clone()` method, `.deepcopy()` clones the data stored in the memory, 
mutating the item returned from `.deepcopy()` will not mutate the item it cloned from.