build:
	tailwindcss -c ./tailwind.config.js -i ./static/main.css -o ./static/styles.css
	cargo build
	
run: build
	cargo run

watch:
	cargo watch -cqs "just run" -i "static/*"