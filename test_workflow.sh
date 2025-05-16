#!/bin/bash

# Exit on error
set -e

# Configuration
SERVER_URL="http://localhost:3000"
PROGRAM_ID="sp1"  # Using the pre-compiled SP1 program
PROOF_FILE="proof_response.json"
VERIFY_FILE="verify_request.json"

# Helper function to make API calls with error handling
make_request() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4
    local data_file=$5

    echo "----------------------------------------"
    echo "$description..."
    
    if [ -n "$data_file" ]; then
        response=$(curl -s -X "$method" "$SERVER_URL/$endpoint" \
            -H "Content-Type: application/json" \
            -d "@$data_file")
    elif [ -n "$data" ]; then
        response=$(curl -s -X "$method" "$SERVER_URL/$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data")
    else
        response=$(curl -s -X "$method" "$SERVER_URL/$endpoint")
    fi

    if [ $? -ne 0 ]; then
        echo "ERROR: Failed to make request to $endpoint"
        exit 1
    fi

    # Validate JSON response
    if ! echo "$response" | jq '.' > /dev/null 2>&1; then
        echo "ERROR: Invalid JSON response from $endpoint"
        exit 1
    fi

    # Handle endpoint-specific response processing
    case "$endpoint" in
        "prove")
            # Save the full response for later use
            echo "$response" > "$PROOF_FILE"
            # Get proof size and proving time
            echo "Proof size: $(jq '.proof | length' "$PROOF_FILE") bytes"
            echo "Proving time: $(jq '.proving_time_milliseconds' "$PROOF_FILE")ms"
            echo "Proof generated successfully (full proof saved to $PROOF_FILE)"
            # Print first 32 bytes of the proof as base64 string
            echo "First 32 bytes of proof (base64): $(jq -r '.proof | .[0:32]' "$PROOF_FILE" | base64)"
            ;;
        "execute")
            # Display execution metrics
            echo "Execution time: $(jq '.execution_time_milliseconds' <<< "$response")ms"
            echo "Total cycles: $(jq '.total_num_cycles' <<< "$response")"
            ;;
        "verify")
            # Check verification result
            if [ "$(jq '.verified' <<< "$response")" = "true" ]; then
                echo "Verification successful"
            else
                echo "Verification failed: $(jq -r '.failure_reason' <<< "$response")"
                exit 1
            fi
            ;;
        *)
            # For other endpoints, print the response
            echo "Response:"
            echo "$response"
            ;;
    esac
}

echo "Starting workflow test..."
echo "========================================"

# Step 1: Get server info
make_request "GET" "info" "" "Getting server information"

# Step 2: Execute the program with supplied values
EXECUTE_DATA="{
    \"program_id\": \"$PROGRAM_ID\",
    \"input\": {
        \"value1\": 10,
        \"value2\": 100
    }
}"
make_request "POST" "execute" "$EXECUTE_DATA" "Executing program"

# Step 3: Generate proof with supplied values
PROVE_DATA="{
    \"program_id\": \"$PROGRAM_ID\",
    \"input\": {
        \"value1\": 10,
        \"value2\": 100
    }
}"
make_request "POST" "prove" "$PROVE_DATA" "Generating proof"

# Step 4: Verify proof
# Create a temporary file for the verification request
if [ -f "$PROOF_FILE" ]; then
    # Create verification request file
    jq -c --arg program_id "$PROGRAM_ID" '{program_id: $program_id, proof: .proof}' "$PROOF_FILE" > "$VERIFY_FILE"
    make_request "POST" "verify" "" "Verifying proof" "$VERIFY_FILE"
    # Clean up temporary files
    rm "$VERIFY_FILE"
    rm "$PROOF_FILE"
else
    echo "Error: $PROOF_FILE not found"
    exit 1
fi

echo "========================================"
echo "Workflow test completed successfully!" 