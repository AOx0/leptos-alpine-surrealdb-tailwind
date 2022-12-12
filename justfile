build:
	tailwindcss -c ./tailwind.config.js -i ./static/main.css -o ./static/styles.css
	cargo clippy 
	
run: build
	cargo run

watch:
	cargo watch -cqs "just run" -i "static/*"