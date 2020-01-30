use common::Tryte;
use common::constants::*;
use common::Result;
use common::Error;

use ternary::IsTryte;

use crate::constants::*;

use std::hash::Hash as StdHash;
use std::hash::Hasher as StdHasher;

macro_rules! implement_debug {
    ($($t:ty),+) => {
    $(
        impl std::fmt::Debug for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0.iter().collect::<String>())
            }
        }
    )+
    }
}

macro_rules! implement_display {
    ($($t:ty),+) => {
    $(
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0.iter().collect::<String>())
            }
        }
    )+
    }
}

macro_rules! implement_hash {
    ($($t:ty),+) => {
    $(
        impl StdHash for $t {
            fn hash<H : StdHasher>(&self,state: &mut H) {
                    self.0.hash(state);
            }
        }
    )+
    }
}

macro_rules! implement_eq {
    ($($t:ty),+) => {
    $(
        impl PartialEq for $t {
            fn eq(&self,other: &$t) -> bool {
                    self.0.iter().zip(other.0.iter()).all(|(a,b)| a == b)
            }
        }

        impl Eq for $t {}

    )+
    }
}

macro_rules! implement_clone {
    ($($t:ty),+) => {
    $(
        impl Clone for $t {
            fn clone(&self) -> Self {
                    let mut cloned : $t = <$t>::default();
                    cloned.0 = self.0;
                    cloned
            }
        }

    )+
    }
}

pub struct Payload(pub [Tryte; PAYLOAD.tryte_offset.length]);
pub struct Address(pub [Tryte; ADDRESS.tryte_offset.length]);
#[derive(Default, Debug)]
pub struct Value(pub i64);
pub struct Tag(pub [Tryte; TAG.tryte_offset.length]);
#[derive(Default, Debug)]
pub struct Timestamp(pub u64);
#[derive(Default, Debug)]
pub struct Index(pub usize);
pub struct Hash(pub [Tryte; BUNDLE_HASH.tryte_offset.length]);
pub struct Nonce(pub [Tryte; NONCE.tryte_offset.length]);

#[derive(Default)]
pub struct Transaction {
    payload: Payload,
    address: Address,
    value: Value,
    obsolete_tag: Tag,
    timestamp: Timestamp,
    index: Index,
    last_index: Index,
    bundle_hash: Hash,
    trunk_hash: Hash,
    branch_hash: Hash,
    tag: Tag,
    attachment_ts: Timestamp,
    attachment_lbts: Timestamp,
    attachment_ubts: Timestamp,
    nonce: Nonce,
}

/// The (bundle) essence of each transaction is a subset of its fields, with a total size of 486 trits, see the table below.
/// NOTE: if this is a subset of transaction fields, then it's confusing to call it the `bundle essence` when in reality it's the essence of a transaction needed to build a bundle, or am I misunderstanding the meaning of the word 'essence'? I would like to call it the `TransactionEssence`, but that might be confusing to people used to IOTA terms.
pub struct Essence<'a> {
    address: &'a Address,
    value: &'a Value,
    obsolete_tag: &'a Tag,
    timestamp: &'a Timestamp,
    index: &'a Index,
    last_index: &'a Index,
}

impl Payload {
    pub fn from_str(payload: &str) -> Self {
        assert!(payload.len() <= PAYLOAD.tryte_offset.length);
        assert!(payload.chars().all(|c| c.is_tryte()));

        let mut trytes = [TRYTE_ZERO; PAYLOAD.tryte_offset.length];

        for (i, c) in payload.chars().enumerate() {
            trytes[i] = c;
        }

        Self(trytes)
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self([TRYTE_ZERO; PAYLOAD.tryte_offset.length])
    }
}

impl Default for Address {
    fn default() -> Self {
        Self([TRYTE_ZERO; ADDRESS.tryte_offset.length])
    }
}

impl Address {
    pub fn from_str(address: &str) -> Self {
        assert!(address.len() <= ADDRESS.tryte_offset.length);
        assert!(address.chars().all(|c| c.is_tryte()));

        let mut trytes = [TRYTE_ZERO; ADDRESS.tryte_offset.length];

        for (i, c) in address.chars().enumerate() {
            trytes[i] = c;
        }

        Self(trytes)
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self([TRYTE_ZERO; TAG.tryte_offset.length])
    }
}

impl Tag {
    pub fn from_str(tag: &str) -> Self {
        assert!(tag.len() <= TAG.tryte_offset.length);
        assert!(tag.chars().all(|c| c.is_tryte()));

        let mut trytes = [TRYTE_ZERO; TAG.tryte_offset.length];

        for (i, c) in tag.chars().enumerate() {
            trytes[i] = c;
        }

        Self(trytes)
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self([TRYTE_ZERO; BUNDLE_HASH.tryte_offset.length])
    }
}

impl Hash {
    pub fn from_str(hash: &str) -> Self {
        assert!(hash.len() <= BUNDLE_HASH.tryte_offset.length);
        assert!(hash.chars().all(|c| c.is_tryte()));

        let mut trytes = [TRYTE_ZERO; BUNDLE_HASH.tryte_offset.length];

        for (i, c) in hash.chars().enumerate() {
            trytes[i] = c;
        }

        Self(trytes)
    }
}


impl Default for Nonce {
    fn default() -> Self {
        Self([TRYTE_ZERO; NONCE.tryte_offset.length])
    }
}

impl Nonce {
    pub fn from_str(nonce: &str) -> Self {
        assert!(nonce.len() <= NONCE.tryte_offset.length);
        assert!(nonce.chars().all(|c| c.is_tryte()));

        let mut trytes = [TRYTE_ZERO; NONCE.tryte_offset.length];

        for (i, c) in nonce.chars().enumerate() {
            trytes[i] = c;
        }

        Self(trytes)
    }
}

implement_debug!(Payload, Address, Tag, Nonce, Hash);
implement_display!(Payload, Address, Tag, Nonce, Hash);
implement_hash!(Address, Hash);
implement_eq!(Payload, Address, Tag, Hash, Nonce);
implement_clone!(Hash);

impl Transaction {
    pub fn from_tryte_str(tx_trytes: &str) -> Self {
        assert_eq!(TRANSACTION_TRYT_LEN, tx_trytes.len());

        /*
        let payload = Payload::from_tx_tryte_str(tx_trytes, PAYLOAD.tryte_offset);

        Self {
            payload,
        }
        */

        unimplemented!()
    }
    /// Create a `Transaction` from a reader object.
    pub fn from_reader<R: std::io::Read>(reader: R) -> Result<Self> {
        Err(Error::TransactionError)
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn obsolete_tag(&self) -> &Tag {
        &self.obsolete_tag
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn last_index(&self) -> &Index {
        &self.last_index
    }

    pub fn bundle_hash(&self) -> &Hash {
        &self.bundle_hash
    }

    pub fn trunk_hash(&self) -> &Hash {
        &self.trunk_hash
    }

    pub fn branch_hash(&self) -> &Hash {
        &self.branch_hash
    }

    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    pub fn attachment_ts(&self) -> &Timestamp {
        &self.attachment_ts
    }

    pub fn attachment_lbts(&self) -> &Timestamp {
        &self.attachment_lbts
    }

    pub fn attachment_ubts(&self) -> &Timestamp {
        &self.attachment_ubts
    }

    pub fn nonce(&self) -> &Nonce {
        &self.nonce
    }

    pub fn essence<'a>(&'a self) -> Essence<'a> {
        Essence {
            address: &self.address,
            value: &self.value,
            obsolete_tag: &self.obsolete_tag,
            timestamp: &self.timestamp,
            index: &self.index,
            last_index: &self.last_index,
        }
    }
}

impl std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "address={:?}\nvalue={:?}\ntimestamp={:?}\nindex={:?}\nlast_index={:?}\ntag={:?}\nbundle_hash={:?}\ntrunk_hash={:?}\nbranch_hash={:?}\nnonce={:?}",
        self.address,
        self.value,
        self.timestamp,
        self.index,
        self.last_index,
        self.tag,
        self.bundle_hash,
        self.trunk_hash,
        self.branch_hash,
        self.nonce)
    }
}

pub struct TransactionBuilder {
    payload: Option<Payload>,
    address: Option<Address>,
    value: Option<Value>,
    obsolete_tag: Option<Tag>,
    timestamp: Option<Timestamp>,
    index: Option<Index>,
    last_index: Option<Index>,
    bundle_hash: Option<Hash>,
    trunk_hash: Option<Hash>,
    branch_hash: Option<Hash>,
    tag: Option<Tag>,
    attachment_ts: Option<Timestamp>,
    attachment_lbts: Option<Timestamp>,
    attachment_ubts: Option<Timestamp>,
    nonce: Option<Nonce>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            payload: None,
            address: None,
            value: None,
            obsolete_tag: None,
            timestamp: None,
            index: None,
            last_index: None,
            tag: None,
            bundle_hash: None,
            trunk_hash: None,
            branch_hash: None,
            attachment_ts: None,
            attachment_lbts: None,
            attachment_ubts: None,
            nonce: None,
        }
    }

    pub fn default() -> Self {
        Self {
            payload: Some(Payload::default()),
            address: Some(Address::default()),
            value: Some(Value::default()),
            obsolete_tag: Some(Tag::default()),
            timestamp: Some(Timestamp::default()),
            index: Some(Index::default()),
            last_index: Some(Index::default()),
            tag: Some(Tag::default()),
            bundle_hash: Some(Hash::default()),
            trunk_hash: Some(Hash::default()),
            branch_hash: Some(Hash::default()),
            attachment_ts: Some(Timestamp::default()),
            attachment_lbts: Some(Timestamp::default()),
            attachment_ubts: Some(Timestamp::default()),
            nonce: Some(Nonce::default()),
        }
    }

    pub fn payload(&mut self, payload: Payload) -> &mut Self {
        self.payload.replace(payload);
        self
    }

    pub fn address(&mut self, address: Address) -> &mut Self {
        self.address.replace(address);
        self
    }

    pub fn value(&mut self, value: Value) -> &mut Self {
        self.value.replace(value);
        self
    }

    pub fn obsolete_tag(&mut self, obsolete_tag: Tag) -> &mut Self {
        self.obsolete_tag.replace(obsolete_tag);
        self
    }
    pub fn timestamp(&mut self, timestamp: Timestamp) -> &mut Self {
        self.timestamp.replace(timestamp);
        self
    }

    pub fn index(&mut self, index: Index) -> &mut Self {
        self.index.replace(index);
        self
    }

    pub fn last_index(&mut self, last_index: Index) -> &mut Self {
        self.last_index.replace(last_index);
        self
    }

    pub fn tag(&mut self, tag: Tag) -> &mut Self {
        self.tag.replace(tag);
        self
    }

    pub fn attachment_ts(&mut self, attachment_ts: Timestamp) -> &mut Self {
        self.attachment_ts.replace(attachment_ts);
        self
    }

    pub fn bundle_hash(&mut self, bundle_hash: Hash) -> &mut Self {
        self.bundle_hash.replace(bundle_hash);
        self
    }

    pub fn trunk_hash(&mut self, trunk_hash: Hash) -> &mut Self {
        self.trunk_hash.replace(trunk_hash);
        self
    }

    pub fn branch_hash(&mut self, branch_hash: Hash) -> &mut Self {
        self.branch_hash.replace(branch_hash);
        self
    }

    pub fn attachment_lbts(&mut self, attachment_lbts: Timestamp) -> &mut Self {
        self.attachment_lbts.replace(attachment_lbts);
        self
    }

    pub fn attachment_ubts(&mut self, attachment_ubts: Timestamp) -> &mut Self {
        self.attachment_ubts.replace(attachment_ubts);
        self
    }

    pub fn nonce(&mut self, nonce: Nonce) -> &mut Self {
        self.nonce.replace(nonce);
        self
    }

    pub fn build(self) -> Transaction {
        Transaction {
            payload: self.payload.unwrap(),
            address: self.address.unwrap(),
            value: self.value.unwrap(),
            obsolete_tag: self.obsolete_tag.unwrap(),
            timestamp: self.timestamp.unwrap(),
            index: self.index.unwrap(),
            last_index: self.last_index.unwrap(),
            tag: self.tag.unwrap(),
            bundle_hash: self.bundle_hash.unwrap(),
            trunk_hash: self.trunk_hash.unwrap(),
            branch_hash: self.branch_hash.unwrap(),
            attachment_ts: self.attachment_ts.unwrap(),
            attachment_lbts: self.attachment_lbts.unwrap(),
            attachment_ubts: self.attachment_ubts.unwrap(),
            nonce: self.nonce.unwrap(),
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;

    #[test]
    fn create_transaction_from_builder() {
        let mut builder = TransactionBuilder::default();
        builder
            .value(Value(10))
            .address(Address::from_str("ME"))
            .tag(Tag::from_str("HELLO"))
            .nonce(Nonce::from_str("ABCDEF"));

        let tx = builder.build();
        println!("{:?}", tx);
    }

    #[test]
    fn create_transaction_from_tryte_string() {
        //let tx = Transaction::from_tryte_str(TX_TRYTES);

    }

    const TX_TRYTES: &str = "SEGQSWYCJHRLJYEGZLRYQAZPLVRAYIWGWJUMFFX99UZUKBQNFYAOQLOFARIKNEBKDRHJJWDJARXTNPHPAODJRSGJBVVYBVJHZALJWDCJHZRSACOVCVVAVHZVTPFTAJWVGFSVLSYXHNNXEGSMJHDBZKGFQNYJJJBAPDHFFGZ9POSOMWTDPGXI9KQRLMUVWNEQDANMXROVORJVALWVGDDJAFOOBXUKVCCIVXSSHZUCZV9XVBASLWX9NXPWGMGYCRD9ILQMKIGPBGGMKAIJKNALBLABATYFVIRBKTXTWNUZAUXRASB9EEIQHWBD9ZYUDBUPBSWXVYXQXECRCHQAYH9ZBUZBASPOIGBSGWJYFKFRITUBVMCYGCMAPTXOIWEVTUXSUOUPTUQOPMMPUTHXMOP9CW9THAZXEPMOMNEOBLUBPOAIOBEBERRZCIKHSTDWUSUPUWNJOCLNZDCEKWWAAJDPJXJEHHSYFN9MH9BGUDQ9CSZBIHRC9PSQJPGKH9ILZDWUWLEKWFKUFFFIMOQKRMKOYXEJHXLCEGCGGKHGJUHOXINSWCKRNMUNAJDCVLZGEBII9ASTYFTDYDZIZSNHIWHSQ9HODQMVNDKMKHCFDXIIGDIVJSBOOE9GRIXCD9ZUTWCUDKFTETSYSRBQABXCXZFOWQMQFXHYZWD9JZXUWHILMRNWXSGUMIIXZYCTWWHCWMSSTCNSQXQXMQPTM9MOQMIVDYNNARDCVNQEDTBKWOIOSKPKPOZHJGJJGNYWQWUWAZMBZJ9XEJMRVRYFQPJ9NOIIXEGIKMMN9DXYQUILRSCSJDIDN9DCTFGQIYWROZQIEQTKMRVLGGDGA9UVZPNRGSVTZYAPMWFUWDEUULSEEGAGITPJQ9DBEYEN9NVJPUWZTOTJHEQIXAPDOICBNNCJVDNM9YRNXMMPCOYHJDUFNCYTZGRCBZKOLHHUK9VOZWHEYQND9WUHDNGFTAS99MRCAU9QOYVUZKTIBDNAAPNEZBQPIRUFUMAWVTCXSXQQIYQPRFDUXCLJNMEIKVAINVCCZROEWEX9XVRM9IHLHQCKC9VLK9ZZWFBJUZKGJCSOPQPFVVAUDLKFJIJKMLZXFBMXLMWRSNDXRMMDLE9VBPUZB9SVLTMHA9DDDANOKIPY9ULDWAKOUDFEDHZDKMU9VMHUSFG9HRGZAZULEJJTEH9SLQDOMZTLVMBCXVNQPNKXRLBOUCCSBZRJCZIUFTFBKFVLKRBPDKLRLZSMMIQNMOZYFBGQFKUJYIJULGMVNFYJWPKPTSMYUHSUEXIPPPPPJTMDQLFFSFJFEPNUBDEDDBPGAOEJGQTHIWISLRDAABO9H9CSIAXPPJYCRFRCIH9TVBZKTCK9SPQZUYMUOKMZYOMPRHRGF9UAKZTZZG9VVVTIHMSNDREUOUOSLKUHTNFXTNSJVPVWCQXUDIMJIAMBPXUGBNDTBYPKYQYJJCDJSCTTWHOJKORLHGKRJMDCMRHSXHHMQBFJWZWHNUHZLYOAFQTRZFXDBYASYKWEVHKYDTJIAUKNCCEPSW9RITZXBOFKBAQOWHKTALQSCHARLUUGXISDMBVEUKOVXTKTEVKLGYVYHPNYWKNLCVETWIHHVTBWT9UPMTQWBZPRPRSISUBIBECVDNIZQULAGLONGVFLVZPBMHJND9CEVIXSYGFZAGGN9MQYOAKMENSEOGCUNKEJTDLEDCD9LGKYANHMZFSSDDZJKTKUJSFL9GYFDICTPJEPDSBXDQTARJQEWUVWDWSQPKIHPJONKHESSQH9FNQEO9WUCFDWPPPTIQPWCVDYTTWPLCJJVYNKE9ZEJNQBEJBMDBLNJKQDOQOHVS9VY9UPSU9KZVDFOESHNRRWBK9EZCYALAUYFGPCEWJQDXFENSNQEAUWDXJGOMCLQUQWMCPHOBZZ9SZJ9KZXSHDLPHPNYMVUJQSQETTN9SG9SIANJHWUYQXZXAJLYHCZYRGITZYQLAAYDVQVNKCDIYWAYBAFBMAYEAEAGMTJGJRSNHBHCEVIQRXEFVWJWOPU9FPDOWIFL9EWGHICRBNRITJDZNYACOGTUDBZYIYZZWAOCDBQFFNTTSTGKECWTVWZSPHX9HNRUYEAEWXENEIDLVVFMZFVPUNHMQPAIOKVIBDIHQIHFGRJOHHONPLGBSJUD9HHDTQQUZN9NVJYOAUMXMMOCNUFLZ9BAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHDQQSQLDHZ9999999999999999999TRINITY99999999999999999999TNXSQ9D99A99999999B99999999MXKZAGDGKVADXOVCAXEQYZGOGQKDLKIUPYXIL9PXYBQXGYDEGNXTFURSWQYLJDFKEV9VVBBQLTLHIBTFYOGBHPUUHS9CKWSAPIMDIRNSUJ9CFPGKTUFAGQYVMFKOZSVAHIFJXWCFBZLICUWF9GNDZWCOWDUIIZ9999OXNRVXLBKJXEZMVABR9UQBVSTBDFSAJVRRNFEJRL9UFTOFPJHQMQKAJHDBIQAETS9OUVTQ9DSPAOZ9999TRINITY99999999999999999999LPZYMWQME999999999MMMMMMMMMDTIZE9999999999999999999999";
}