set dotenv-load

playlist id:
  cargo run -- playlist --playlist-id {{id}} --out playlist_{{id}}.ron

create_mix id:
  cargo run -- create-mix playlist_{{id}}.ron --out {{id}}.mp3