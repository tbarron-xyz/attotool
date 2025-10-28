FROM rust:latest

WORKDIR /app

# Copy all project files
COPY . .

# Build the application in debug mode
RUN cargo build

# Expose port 7999
EXPOSE 7999

# Set the entrypoint to the built binary
ENTRYPOINT ["./target/debug/attotool-rs"]