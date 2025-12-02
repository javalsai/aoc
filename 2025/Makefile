tester: tester.rs
	rustc --edition 2024 $< \
		-O -Copt-level=3 -Cstrip=symbols \
		-Cdebuginfo=0 -Cdebug-assertions=off \
		-Coverflow-checks=false -Cpanic=abort \
		-Ctarget-cpu=native \
		-Ccodegen-units=1
