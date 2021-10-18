for f in ./test_code/*.krt; do
    # do some stuff here with "$f"
    # remember to quote it or spaces may misbehave
    echo "\n+++++++++++++++++++++++++++++++++++++++++"
    echo "RUNNING "$f""
    if [ "$1" = "binary" ]; then
        target/debug/krusty "$f" || exit 1
    else
        cargo run "$f" || exit 1
    fi
    echo "+++++++++++++++++++++++++++++++++++++++++\n"
    # sleep 1
done

echo "success!!!"
exit 0
