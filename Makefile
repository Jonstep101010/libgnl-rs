# make sure different buffer size library targets are cached
# BUFFER_SIZEs are 1, 10, 42, 1000000, 10000000
BUILD_DIR = ./target/release
TEST_TARGETS = ./test_targets
define BUILD_TARGET
$(TEST_TARGETS)/libget_next_line_$(1).a:
	@(BUFFER_SIZE=$(1) cargo build --release)
	@cp $(BUILD_DIR)/libget_next_line.a $(TEST_TARGETS)/libget_next_line_$(1).a
endef

$(eval $(call BUILD_TARGET,1))
$(eval $(call BUILD_TARGET,10))
$(eval $(call BUILD_TARGET,42))
$(eval $(call BUILD_TARGET,1000000))
# cargo will break if trying with 10000000

# invoke gnlTester, fsoares
prep: $(TEST_TARGETS)/libget_next_line_1.a $(TEST_TARGETS)/libget_next_line_10.a $(TEST_TARGETS)/libget_next_line_42.a $(TEST_TARGETS)/libget_next_line_1000000.a

test: prep
	rm -rf c-gnltests/tests/$(TEST_TARGETS)/*.a
	cp -r $(TEST_TARGETS) c-gnltests/tests/
	$(MAKE) -C c-gnltests/tests/gnlTester m
	$(MAKE) -C c-gnltests/tests/fsoares mandatory

clean:
	rm -rf $(TEST_TARGETS)
	mkdir -p $(TEST_TARGETS)

re: clean test
