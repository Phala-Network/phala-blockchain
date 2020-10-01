cd /root && \
wget https://apt.llvm.org/llvm.sh && \
chmod +x llvm.sh && \
./llvm.sh 10 && \
rm ./llvm.sh && \
rm -rf /var/lib/apt/lists/*
