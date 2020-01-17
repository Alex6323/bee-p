/// `Bundle`s are messages on the network of one or more `Transactions`s, which in turn are setnt one at a time and are stored in a distributed ledger called the `Tangle`.
///
/// For a `Bundle` to be bulidable, all required transactions have to be present when validating and building. Otherwise the build will fail.
struct Bundle;

/// Concerned with constructing and verifying complete messages coming in externally.
struct IncomingBundleBuilder;

/// Responsible for constructing a `Bundle` from scratch to be sent to the IOTA network. This includes siging its transactions, calculating the bundle hash, and setting other releveant fields depending on context.
struct OutgoingBundleBuilder;
struct SealedBundleBuilder;
struct SignedBundleBuilder;

struct AttachedBundleBuilder;
struct ValidatedBundleBuilder;
