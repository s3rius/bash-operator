# Kubernetes operator framework for BASH

This simple project will help you to easily write scripts for tracking resources
of your cluster using bash scripts. 


Here's an example of an operator that adds finalizers and removes them when resource is being deleted.
```bash
#!/bin/bash
# Don't use set -e, otherwise cleanup process might
# not cleanup all jobs.
# set -e

# This is a simple example of how to use the bash-operator.
function my_watcher(){
  ACTION="$1"
  MANIFEST="$2"
  REQUEUE_FILE="$3"

  if [[ "$ACTION" == "Apply" ]]; then
    echo "Updated" "$(jq -r '.metadata.name' "$MANIFEST")"
    ./target/debug/bash-operator utils add-finalizer "bop/finalizer" "$MANIFEST"
  fi

  if [[ "$ACTION" == "Delete" ]]; then
    echo "Removed" "$(jq -r '.metadata.name' "$MANIFEST")"
    ./target/debug/bash-operator utils remove-finalizer "bop/finalizer" "$MANIFEST"
  fi

  # Required to update after 10 seconds.
  echo "10" > "$REQUEUE_FILE"
}

export -f my_watcher

# Start the operator for each resource type.
./target/debug/bash-operator operator "v1/Secret" my_watcher &
./target/debug/bash-operator operator --namespace "kube-system" "apps/v1/Secret" my_watcher &
./target/debug/bash-operator operator --all-namespaces "v1/Pod" my_watcher &

# Function to cleanup the background jobs.
# This is called when the script exits.
function cleanup() {
  echo "Cleaning up..."
  kill `jobs -p`
}

# This is a trap to kill all the background jobs when the script exits.
trap cleanup SIGINT SIGTERM EXIT
# This is to keep the script running.
wait < <(jobs -p)
```

So, here's the main idea behind it. First of all, one instance of an operator can handle only one resource type. 
If you need to track multiple resources, start operators for each resource type in background using `&`.

Bash functions should be exported using `export -f <func_name>`. Because otherwise this function won't be available for calling inside the operator process.

The operator executes given function with 3 arguments. 
* Action that triggered the function;
* Path to a file with full definition of an object;
* Path to a file where you can put any requeue duration for this object.

Actions can be either `Apply` or `Delete`. And in case if the object doesn't contain any finalizers, the watcher won't
 be able to determine if object is being deleted. So, if you want to track deletions, please use finalizers as shown in the example.
