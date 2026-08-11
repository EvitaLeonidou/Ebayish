# Use an official Node.js runtime as a parent image
FROM node:22-alpine

# Install necessary system dependencies for mkcert
RUN apk add --no-cache ca-certificates

# Set the working directory in the container
WORKDIR /app

# Copy package.json and package-lock.json (or yarn.lock, etc.)
COPY package*.json ./

# Install project dependencies
RUN npm install

# Copy the rest of the application code into the container
COPY . .

# Create directory for certificates
RUN mkdir -p /app/.vite

# The Vite server needs to be exposed on all network interfaces inside the container
ENV HOST=0.0.0.0

# Make port available to the world outside this container
EXPOSE 5173

# Run the app when the container launches
# Use "npm run dev -- --host" to ensure Vite listens on all network interfaces
CMD ["npm", "run", "dev", "--", "--host"]
