for f in ./test_code/*.kry; do
    # do some stuff here with "$f"
    # remember to quote it or spaces may misbehave
    echo "\n+++++++++++++++++++++++++++++++++++++++++"
    echo "RUNNING "$f""
    cargo run "$f"
    echo "+++++++++++++++++++++++++++++++++++++++++\n"
    # sleep 1
done
