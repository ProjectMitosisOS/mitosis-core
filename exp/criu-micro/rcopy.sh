if [ "$#" -ne 2 ]; then
    echo "Usage: $0 directory_path ipoib_ip"
    exit 1
fi

for f in $(ls $1)
do
  rcopy $1/$f $2
done
