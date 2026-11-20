// Minimal OS stub so that `random_pcg.cpp` (pulled in by `math_funcs.cpp`)
// links into the parity harness. `RandomPCG::randomize()` is never called by
// the harness, so returning the (never created) singleton is fine.

#include "core/os/os.h"

OS *OS::singleton = nullptr;

OS *OS::get_singleton() {
	return singleton;
}
