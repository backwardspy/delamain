FROM scratch
COPY target/x86_64-unknown-linux-musl/release/delamain .
CMD ["./delamain"]
