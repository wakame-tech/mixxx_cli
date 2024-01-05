set dotenv-load

playlist id:
  cargo run -- playlist --playlist-id {{id}} --out mix_{{id}}.csv

create_mix csv:
  cargo run -- create-mix {{csv}} --out out.mp3