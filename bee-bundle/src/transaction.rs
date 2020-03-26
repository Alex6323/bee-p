use crate::constants::{
    ADDRESS,
    ADDRESS_TRIT_LEN,
    BRANCH,
    BUNDLE,
    HASH_TRIT_LEN,
    IOTA_SUPPLY,
    NONCE,
    NONCE_TRIT_LEN,
    OBSOLETE_TAG,
    PAYLOAD,
    PAYLOAD_TRIT_LEN,
    TAG,
    TAG_TRIT_LEN,
    TRUNK,
    TRYTE_ZERO,
};

use bee_ternary::{
    raw::RawEncoding,
    raw::RawEncodingBuf,
    Btrit,
    T1B1Buf,
    Trit,
    TritBuf,
    Trits,
};

use std::fmt;
use std::hash;
use std::iter;

#[derive(Debug)]
pub enum TransactionFieldError {
    FieldWrongLength,
    FieldDeserializationError,
}

pub trait TransactionField: Sized {
    type inner;
    fn try_from_inner(buffer: Self::inner) -> Result<Self, TransactionFieldError>;
    fn from_inner_unchecked(buffer: Self::inner) -> Self;

    fn into_inner(&self) -> &Self::inner;

    fn trit_len() -> usize;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Payload(TritBuf<T1B1Buf>);

impl Payload {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(PAYLOAD.trit_offset.length))
    }

    pub fn as_bytes(&self) -> &[i8] {
        self.0.as_i8_slice()
    }

    pub fn trit_len() -> usize {
        PAYLOAD_TRIT_LEN
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Address(TritBuf<T1B1Buf>);

impl Address {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(ADDRESS.trit_offset.length))
    }

    pub fn as_bytes(&self) -> &[i8] {
        self.0.as_i8_slice()
    }

    pub fn trit_len() -> usize {
        ADDRESS_TRIT_LEN
    }
}

impl Eq for Address {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Value(i64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag(TritBuf<T1B1Buf>);

impl Tag {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(TAG.trit_offset.length))
    }

    pub fn as_bytes(&self) -> &[i8] {
        self.0.as_i8_slice()
    }

    pub fn trit_len() -> usize {
        TAG_TRIT_LEN
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timestamp(u64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Index(usize);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hash(TritBuf<T1B1Buf>);

impl Hash {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(BUNDLE.trit_offset.length))
    }

    pub fn as_bytes(&self) -> &[i8] {
        self.0.as_i8_slice()
    }

    pub fn trit_len() -> usize {
        HASH_TRIT_LEN
    }
}

impl Eq for Hash {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Nonce(TritBuf<T1B1Buf>);

impl Nonce {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(NONCE.trit_offset.length))
    }

    pub fn as_bytes(&self) -> &[i8] {
        self.0.as_i8_slice()
    }

    pub fn trit_len() -> usize {
        NONCE_TRIT_LEN
    }
}

macro_rules! impl_into_inner_for_trit_fields {
    ( $($field_name:ident),+ $(,)?) => {
        $(
            impl TransactionField for $field_name {

                type inner = TritBuf<T1B1Buf>;
                fn into_inner(&self) -> &Self::inner{
                    &self.0
                }

                fn try_from_inner(buffer: Self::inner) -> Result<Self, TransactionFieldError> {
                    if buffer.len() != $field_name::trit_len() {
                        Err(TransactionFieldError::FieldWrongLength)?
                    }

                    Ok(Self::from_inner_unchecked(buffer))
                }

                fn from_inner_unchecked(buffer: Self::inner) -> Self{

                    Self(buffer)
                }

                fn trit_len() -> usize {
                   Self::trit_len()
                }
            }
        )+
    }
}

impl TransactionField for Index {
    type inner = usize;
    fn into_inner(&self) -> &Self::inner {
        &self.0
    }

    fn try_from_inner(inner: Self::inner) -> Result<Self, TransactionFieldError> {
        Ok(Self(inner))
    }

    fn from_inner_unchecked(inner: Self::inner) -> Self {
        Self(inner)
    }

    fn trit_len() -> usize {
        unimplemented!()
    }
}

impl TransactionField for Value {
    type inner = i64;
    fn into_inner(&self) -> &Self::inner {
        &self.0
    }

    fn try_from_inner(inner: Self::inner) -> Result<Self, TransactionFieldError> {
        Ok(Self(inner))
    }

    fn from_inner_unchecked(inner: Self::inner) -> Self {
        Self(inner)
    }

    fn trit_len() -> usize {
        unimplemented!()
    }
}

impl TransactionField for Timestamp {
    type inner = u64;
    fn into_inner(&self) -> &Self::inner {
        &self.0
    }

    fn try_from_inner(inner: Self::inner) -> Result<Self, TransactionFieldError> {
        Ok(Self(inner))
    }

    fn from_inner_unchecked(inner: Self::inner) -> Self {
        Self(inner)
    }

    fn trit_len() -> usize {
        unimplemented!()
    }
}

macro_rules! impl_hash_trait {
    ( $($field_name:ident),+ $(,)?) => {
        $(
            impl hash::Hash for $field_name {
                fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
                       self.0.hash(hasher)
                }
            }
        )+
    }
}

impl_into_inner_for_trit_fields!(Payload, Address, Hash, Tag, Nonce);
impl_hash_trait!(Hash, Address);

#[derive(Debug)]
pub enum TransactionError {
    TransactionDeserializationError,
    TransactionBuilderError(TransactionBuilderError),
}

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub(crate) payload: Payload,
    pub(crate) address: Address,
    pub(crate) value: Value,
    pub(crate) obsolete_tag: Tag,
    pub(crate) timestamp: Timestamp,
    pub(crate) index: Index,
    pub(crate) last_index: Index,
    pub(crate) bundle: Hash,
    pub(crate) trunk: Hash,
    pub(crate) branch: Hash,
    pub(crate) tag: Tag,
    pub(crate) attachment_ts: Timestamp,
    pub(crate) attachment_lbts: Timestamp,
    pub(crate) attachment_ubts: Timestamp,
    pub(crate) nonce: Nonce,
}

impl Transaction {
    pub fn builder() -> TransactionBuilder {
        TransactionBuilder::new()
    }

    pub fn from_trits(buffer: &Trits<impl RawEncoding<Trit = Btrit> + ?Sized>) -> Result<Self, TransactionError> {
        let trits = buffer.encode::<T1B1Buf>();

        let transaction = Self::builder()
            .with_payload(Payload(
                trits[PAYLOAD.trit_offset.start..PAYLOAD.trit_offset.start + PAYLOAD.trit_offset.length].to_buf(),
            ))
            .with_address(Address(
                trits[ADDRESS.trit_offset.start..ADDRESS.trit_offset.start + ADDRESS.trit_offset.length].to_buf(),
            ))
            .with_value(Value::from_inner_unchecked(0))
            .with_obsolete_tag(Tag(trits[OBSOLETE_TAG.trit_offset.start
                ..OBSOLETE_TAG.trit_offset.start + OBSOLETE_TAG.trit_offset.length]
                .to_buf()))
            .with_timestamp(Timestamp::from_inner_unchecked(0))
            .with_index(Index::from_inner_unchecked(0))
            .with_last_index(Index::from_inner_unchecked(0))
            .with_tag(Tag(trits
                [TAG.trit_offset.start..TAG.trit_offset.start + TAG.trit_offset.length]
                .to_buf()))
            .with_attachment_ts(Timestamp::from_inner_unchecked(0))
            .with_bundle(Hash(
                trits[BUNDLE.trit_offset.start..BUNDLE.trit_offset.start + BUNDLE.trit_offset.length].to_buf(),
            ))
            .with_trunk(Hash(
                trits[TRUNK.trit_offset.start..TRUNK.trit_offset.start + TRUNK.trit_offset.length].to_buf(),
            ))
            .with_branch(Hash(
                trits[BRANCH.trit_offset.start..BRANCH.trit_offset.start + BRANCH.trit_offset.length].to_buf(),
            ))
            .with_attachment_lbts(Timestamp::from_inner_unchecked(0))
            .with_attachment_ubts(Timestamp::from_inner_unchecked(0))
            .with_nonce(Nonce(
                trits[NONCE.trit_offset.start..NONCE.trit_offset.start + NONCE.trit_offset.length].to_buf(),
            ))
            .build()
            .map_err(|e| TransactionError::TransactionBuilderError(e))?;

        Ok(transaction)
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

    pub fn bundle(&self) -> &Hash {
        &self.bundle
    }

    pub fn trunk(&self) -> &Hash {
        &self.trunk
    }

    pub fn branch(&self) -> &Hash {
        &self.branch
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
}

#[derive(Default)]
pub struct Transactions(pub(crate) Vec<Transaction>);

impl Transactions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, index: usize) -> Option<&Transaction> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, transaction: Transaction) {
        self.0.push(transaction);
    }
}

/// Milestone - TODO - builder?

#[derive(Debug, Clone)]
pub struct Milestone {
    pub hash: Hash,
    pub index: u32,
}

/// Transaction builder

#[derive(Debug)]
pub enum TransactionBuilderError {
    MissingField(&'static str),
    InvalidValue(i64),
}

#[derive(Default)]
pub struct TransactionBuilder {
    pub(crate) payload: Option<Payload>,
    pub(crate) address: Option<Address>,
    pub(crate) value: Option<Value>,
    pub(crate) obsolete_tag: Option<Tag>,
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) index: Option<Index>,
    pub(crate) last_index: Option<Index>,
    pub(crate) bundle: Option<Hash>,
    pub(crate) trunk: Option<Hash>,
    pub(crate) branch: Option<Hash>,
    pub(crate) tag: Option<Tag>,
    pub(crate) attachment_ts: Option<Timestamp>,
    pub(crate) attachment_lbts: Option<Timestamp>,
    pub(crate) attachment_ubts: Option<Timestamp>,
    pub(crate) nonce: Option<Nonce>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload.replace(payload);
        self
    }

    pub fn with_address(mut self, address: Address) -> Self {
        self.address.replace(address);
        self
    }

    pub fn with_value(mut self, value: Value) -> Self {
        self.value.replace(value);
        self
    }

    pub fn with_obsolete_tag(mut self, obsolete_tag: Tag) -> Self {
        self.obsolete_tag.replace(obsolete_tag);
        self
    }
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp.replace(timestamp);
        self
    }

    pub fn with_index(mut self, index: Index) -> Self {
        self.index.replace(index);
        self
    }

    pub fn with_last_index(mut self, last_index: Index) -> Self {
        self.last_index.replace(last_index);
        self
    }

    pub fn with_tag(mut self, tag: Tag) -> Self {
        self.tag.replace(tag);
        self
    }

    pub fn with_attachment_ts(mut self, attachment_ts: Timestamp) -> Self {
        self.attachment_ts.replace(attachment_ts);
        self
    }

    pub fn with_bundle(mut self, bundle: Hash) -> Self {
        self.bundle.replace(bundle);
        self
    }

    pub fn with_trunk(mut self, trunk: Hash) -> Self {
        self.trunk.replace(trunk);
        self
    }

    pub fn with_branch(mut self, branch: Hash) -> Self {
        self.branch.replace(branch);
        self
    }

    pub fn with_attachment_lbts(mut self, attachment_lbts: Timestamp) -> Self {
        self.attachment_lbts.replace(attachment_lbts);
        self
    }

    pub fn with_attachment_ubts(mut self, attachment_ubts: Timestamp) -> Self {
        self.attachment_ubts.replace(attachment_ubts);
        self
    }

    pub fn with_nonce(mut self, nonce: Nonce) -> Self {
        self.nonce.replace(nonce);
        self
    }

    pub fn build(self) -> Result<Transaction, TransactionBuilderError> {
        let value = self
            .value
            .as_ref()
            .ok_or(TransactionBuilderError::MissingField("value"))?
            .0;

        if value.abs() > IOTA_SUPPLY {
            Err(TransactionBuilderError::InvalidValue(value))?;
        }

        Ok(Transaction {
            payload: self.payload.ok_or(TransactionBuilderError::MissingField("payload"))?,
            address: self.address.ok_or(TransactionBuilderError::MissingField("address"))?,
            value: Value(value),
            obsolete_tag: self
                .obsolete_tag
                .ok_or(TransactionBuilderError::MissingField("obsolete_tag"))?,
            timestamp: self
                .timestamp
                .ok_or(TransactionBuilderError::MissingField("timestamp"))?,
            index: self.index.ok_or(TransactionBuilderError::MissingField("index"))?,
            last_index: self
                .last_index
                .ok_or(TransactionBuilderError::MissingField("last_index"))?,
            tag: self.tag.ok_or(TransactionBuilderError::MissingField("tag"))?,
            bundle: self.bundle.ok_or(TransactionBuilderError::MissingField("bundle"))?,
            trunk: self.trunk.ok_or(TransactionBuilderError::MissingField("trunk"))?,
            branch: self.branch.ok_or(TransactionBuilderError::MissingField("branch"))?,
            attachment_ts: self
                .attachment_ts
                .ok_or(TransactionBuilderError::MissingField("attachment_ts"))?,
            attachment_lbts: self
                .attachment_lbts
                .ok_or(TransactionBuilderError::MissingField("attachment_lbts"))?,
            attachment_ubts: self
                .attachment_ubts
                .ok_or(TransactionBuilderError::MissingField("attachment_ubts"))?,
            nonce: self.nonce.ok_or(TransactionBuilderError::MissingField("nonce"))?,
        })
    }
}

#[derive(Default)]
pub struct TransactionBuilders(pub(crate) Vec<TransactionBuilder>);

impl TransactionBuilders {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, transaction_builder: TransactionBuilder) {
        self.0.push(transaction_builder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_transaction_from_builder() {
        let _ = TransactionBuilder::new()
            .with_payload(Payload::zeros())
            .with_address(Address::zeros())
            .with_value(Value(0))
            .with_obsolete_tag(Tag::zeros())
            .with_timestamp(Timestamp(0))
            .with_index(Index(0))
            .with_last_index(Index(0))
            .with_tag(Tag::zeros())
            .with_attachment_ts(Timestamp(0))
            .with_bundle(Hash::zeros())
            .with_trunk(Hash::zeros())
            .with_branch(Hash::zeros())
            .with_attachment_lbts(Timestamp(0))
            .with_attachment_ubts(Timestamp(0))
            .with_nonce(Nonce::zeros())
            .build()
            .unwrap();
    }

    const TX_TRYTES: &str = "SEGQSWYCJHRLJYEGZLRYQAZPLVRAYIWGWJUMFFX99UZUKBQNFYAOQLOFARIKNEBKDRHJJWDJARXTNPHPAODJRSGJBVVYBVJHZALJWDCJHZRSACOVCVVAVHZVTPFTAJWVGFSVLSYXHNNXEGSMJHDBZKGFQNYJJJBAPDHFFGZ9POSOMWTDPGXI9KQRLMUVWNEQDANMXROVORJVALWVGDDJAFOOBXUKVCCIVXSSHZUCZV9XVBASLWX9NXPWGMGYCRD9ILQMKIGPBGGMKAIJKNALBLABATYFVIRBKTXTWNUZAUXRASB9EEIQHWBD9ZYUDBUPBSWXVYXQXECRCHQAYH9ZBUZBASPOIGBSGWJYFKFRITUBVMCYGCMAPTXOIWEVTUXSUOUPTUQOPMMPUTHXMOP9CW9THAZXEPMOMNEOBLUBPOAIOBEBERRZCIKHSTDWUSUPUWNJOCLNZDCEKWWAAJDPJXJEHHSYFN9MH9BGUDQ9CSZBIHRC9PSQJPGKH9ILZDWUWLEKWFKUFFFIMOQKRMKOYXEJHXLCEGCGGKHGJUHOXINSWCKRNMUNAJDCVLZGEBII9ASTYFTDYDZIZSNHIWHSQ9HODQMVNDKMKHCFDXIIGDIVJSBOOE9GRIXCD9ZUTWCUDKFTETSYSRBQABXCXZFOWQMQFXHYZWD9JZXUWHILMRNWXSGUMIIXZYCTWWHCWMSSTCNSQXQXMQPTM9MOQMIVDYNNARDCVNQEDTBKWOIOSKPKPOZHJGJJGNYWQWUWAZMBZJ9XEJMRVRYFQPJ9NOIIXEGIKMMN9DXYQUILRSCSJDIDN9DCTFGQIYWROZQIEQTKMRVLGGDGA9UVZPNRGSVTZYAPMWFUWDEUULSEEGAGITPJQ9DBEYEN9NVJPUWZTOTJHEQIXAPDOICBNNCJVDNM9YRNXMMPCOYHJDUFNCYTZGRCBZKOLHHUK9VOZWHEYQND9WUHDNGFTAS99MRCAU9QOYVUZKTIBDNAAPNEZBQPIRUFUMAWVTCXSXQQIYQPRFDUXCLJNMEIKVAINVCCZROEWEX9XVRM9IHLHQCKC9VLK9ZZWFBJUZKGJCSOPQPFVVAUDLKFJIJKMLZXFBMXLMWRSNDXRMMDLE9VBPUZB9SVLTMHA9DDDANOKIPY9ULDWAKOUDFEDHZDKMU9VMHUSFG9HRGZAZULEJJTEH9SLQDOMZTLVMBCXVNQPNKXRLBOUCCSBZRJCZIUFTFBKFVLKRBPDKLRLZSMMIQNMOZYFBGQFKUJYIJULGMVNFYJWPKPTSMYUHSUEXIPPPPPJTMDQLFFSFJFEPNUBDEDDBPGAOEJGQTHIWISLRDAABO9H9CSIAXPPJYCRFRCIH9TVBZKTCK9SPQZUYMUOKMZYOMPRHRGF9UAKZTZZG9VVVTIHMSNDREUOUOSLKUHTNFXTNSJVPVWCQXUDIMJIAMBPXUGBNDTBYPKYQYJJCDJSCTTWHOJKORLHGKRJMDCMRHSXHHMQBFJWZWHNUHZLYOAFQTRZFXDBYASYKWEVHKYDTJIAUKNCCEPSW9RITZXBOFKBAQOWHKTALQSCHARLUUGXISDMBVEUKOVXTKTEVKLGYVYHPNYWKNLCVETWIHHVTBWT9UPMTQWBZPRPRSISUBIBECVDNIZQULAGLONGVFLVZPBMHJND9CEVIXSYGFZAGGN9MQYOAKMENSEOGCUNKEJTDLEDCD9LGKYANHMZFSSDDZJKTKUJSFL9GYFDICTPJEPDSBXDQTARJQEWUVWDWSQPKIHPJONKHESSQH9FNQEO9WUCFDWPPPTIQPWCVDYTTWPLCJJVYNKE9ZEJNQBEJBMDBLNJKQDOQOHVS9VY9UPSU9KZVDFOESHNRRWBK9EZCYALAUYFGPCEWJQDXFENSNQEAUWDXJGOMCLQUQWMCPHOBZZ9SZJ9KZXSHDLPHPNYMVUJQSQETTN9SG9SIANJHWUYQXZXAJLYHCZYRGITZYQLAAYDVQVNKCDIYWAYBAFBMAYEAEAGMTJGJRSNHBHCEVIQRXEFVWJWOPU9FPDOWIFL9EWGHICRBNRITJDZNYACOGTUDBZYIYZZWAOCDBQFFNTTSTGKECWTVWZSPHX9HNRUYEAEWXENEIDLVVFMZFVPUNHMQPAIOKVIBDIHQIHFGRJOHHONPLGBSJUD9HHDTQQUZN9NVJYOAUMXMMOCNUFLZ9BAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHDQQSQLDHZ9999999999999999999TRINITY99999999999999999999TNXSQ9D99A99999999B99999999MXKZAGDGKVADXOVCAXEQYZGOGQKDLKIUPYXIL9PXYBQXGYDEGNXTFURSWQYLJDFKEV9VVBBQLTLHIBTFYOGBHPUUHS9CKWSAPIMDIRNSUJ9CFPGKTUFAGQYVMFKOZSVAHIFJXWCFBZLICUWF9GNDZWCOWDUIIZ9999OXNRVXLBKJXEZMVABR9UQBVSTBDFSAJVRRNFEJRL9UFTOFPJHQMQKAJHDBIQAETS9OUVTQ9DSPAOZ9999TRINITY99999999999999999999LPZYMWQME999999999MMMMMMMMMDTIZE9999999999999999999999";
}
