for f in ./test_code/*.kry; do
    # do some stuff here with "$f"
    # remember to quote it or spaces may misbehave
    echo "\n+++++++++++++++++++++++++++++++++++++++++"
    echo "RUNNING "$f""
    if [ $1 = "binary" ]; then
        target/debug/krusty "$f"
    else
        cargo run "$f"
    fi
    echo "+++++++++++++++++++++++++++++++++++++++++\n"
    # sleep 1
done
