//! `SignedPacket` message content. The message may be linked to any other message
//! in the channel. It contains both plain and masked payloads. The message can only
//! be signed and published by channel owner. Channel owner must firstly publish
//! corresponding public key certificate in either `Announce` or `ChangeKey` message.
//!
//! ```pb3
//! message SignedPacket {
//!     join link msgid;
//!     absorb trytes public_payload;
//!     mask trytes masked_payload;
//!     commit;
//!     squeeze external tryte hash[78];
//!     mssig(hash) sig;
//! }
//! ```
//!
//! # Fields
//!
//! * `msgid` -- link to the base message.
//!
//! * `public_payload` -- public part of payload.
//!
//! * `masked_payload` -- masked part of payload.
//!
//! * `hash` -- hash value to be signed.
//!
//! * `sig` -- message signature generated with one of channel owner's private key.
//!

use crate::app::core::{MsgId, MSGID_SIZE};
use crate::mss;
use crate::pb3::{self, Absorb, Mask, Result};
use crate::spongos::Spongos;
use crate::trits::{TritSlice, TritSliceMut};

/// Type of `SignedPacket` message content.
pub const TYPE: &str = "MAM9SIGNEDPACKET";

/// Size of `SignedPacket` message content.
///
/// # Arguments
///
/// * `public_trytes` -- size of public payload in trytes.
///
/// * `masked_trytes` -- size of masked payload in trytes.
///
/// * `sk` -- channel owner's MSS private key.
pub fn sizeof(public_trytes: usize, masked_trytes: usize, sk: &mss::PrivateKey) -> usize {
    0
    // join link msgid;
        + pb3::sizeof_ntrytes(MSGID_SIZE / 3)
    // absorb trytes public_payload;
        + pb3::sizeof_trytes(public_trytes)
    // mask trytes masked_payload;
        + pb3::sizeof_trytes(masked_trytes)
    // mssig;
        + pb3::mssig::sizeof_mssig(sk)
}

/// Wrap `SignedPacket` content.
///
/// # Arguments
///
/// * `msgid` -- link to the base message.
///
/// * `slink` -- spongos instance of the message linked by `msgid`.
///
/// * `public_payload` -- public payload.
///
/// * `masked_payload` -- masked payload.
///
/// * `sk` -- channel owner's MSS private key.
///
/// * `s` -- current spongos instance.
///
/// * `b` -- output buffer.
pub fn wrap(
    msgid: &MsgId,
    slink: &mut Spongos,
    public_payload: &pb3::Trytes,
    masked_payload: &pb3::Trytes,
    sk: &mss::PrivateKey,
    s: &mut Spongos,
    b: &mut TritSliceMut,
) {
    assert!(public_payload.size() % 3 == 0);
    assert!(masked_payload.size() % 3 == 0);
    pb3::join::wrap_join(msgid.id.slice(), slink, s, b);
    public_payload.wrap_absorb(s, b);
    masked_payload.wrap_mask(s, b);
    pb3::mssig::squeeze_wrap_mssig(sk, s, b);
}

/// Unwrap `SignedPacket` content and recover signer's MSS public key.
///
/// # Arguments
///
/// * `lookup_link` -- lookup function taking `msgid` as input and returning
/// spongos instance.
///
/// * `s` -- current spongos instance.
///
/// * `b` -- output buffer.
///
/// # Return
///
/// A tuple of public and masked payloads or error code.
pub fn unwrap_recover(
    lookup_link: impl Fn(TritSlice) -> Option<(Spongos, ())>,
    s: &mut Spongos,
    b: &mut TritSlice,
) -> Result<(mss::PublicKey, pb3::Trytes, pb3::Trytes)> {
    pb3::join::unwrap_join(lookup_link, s, b)?;
    let public_payload = pb3::Trytes::unwrap_absorb_sized(s, b)?;
    let masked_payload = pb3::Trytes::unwrap_mask_sized(s, b)?;
    let mss_pk = pb3::mssig::squeeze_unwrap_mssig_recover(s, b)?;
    Ok((mss_pk, public_payload, masked_payload))
}

/// Unwrap `SignedPacket` content and verify signature.
///
/// # Arguments
///
/// * `lookup_link` -- lookup function taking `msgid` as input and returning
/// spongos instance.
///
/// * `mss_pk` -- channel owner's MSS public key.
///
/// * `s` -- current spongos instance.
///
/// * `b` -- output buffer.
///
/// # Return
///
/// A pair of public and masked payloads or error code.
pub fn unwrap_verify(
    lookup_link: impl Fn(TritSlice) -> Option<(Spongos, ())>,
    mss_pk: &mss::PublicKey,
    s: &mut Spongos,
    b: &mut TritSlice,
) -> Result<(pb3::Trytes, pb3::Trytes)> {
    pb3::join::unwrap_join(lookup_link, s, b)?;
    let public_payload = pb3::Trytes::unwrap_absorb_sized(s, b)?;
    let masked_payload = pb3::Trytes::unwrap_mask_sized(s, b)?;
    pb3::mssig::squeeze_unwrap_mssig_verify(mss_pk, s, b)?;
    Ok((public_payload, masked_payload))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::app::channel::msg;
    use crate::prng;
    use crate::trits::Trits;

    #[test]
    fn wrap_unwrap() {
        // secrets, nonces
        let mss_nonce = Trits::from_str("MSSNONCE").unwrap();

        // secret objects
        let prng = prng::dbg_init_str("PRNGKEY");
        let d = 2;
        let mss_sk = mss::PrivateKey::gen(&prng, mss_nonce.slice(), d);
        // data objects
        let msgid = MsgId {
            id: Trits::cycle_str(81, "MSGID"),
        };
        let public_payload = pb3::Trytes(Trits::cycle_str(555, "PUBLIC9PAYLOAD"));
        let masked_payload = pb3::Trytes(Trits::cycle_str(444, "MASKED9PAYLOAD"));

        // message
        let n = msg::signed_packet::sizeof(
            public_payload.size() / 3,
            masked_payload.size() / 3,
            &mss_sk,
        );
        let mut buf = Trits::zero(n);

        // wrap
        {
            let mut s = Spongos::init();
            let mut b = buf.slice_mut();
            let mut slink = Spongos::init();
            msg::signed_packet::wrap(
                &msgid,
                &mut slink,
                &public_payload,
                &masked_payload,
                &mss_sk,
                &mut s,
                &mut b,
            );
            assert_eq!(0, b.size());
        }

        // unwrap
        {
            let mut s = Spongos::init();
            let mut b = buf.slice();
            let slink = Spongos::init();
            let r = msg::signed_packet::unwrap_verify(
                |m| {
                    if m == msgid.id.slice() {
                        Some((slink.clone(), ()))
                    } else {
                        None
                    }
                },
                mss_sk.public_key(),
                &mut s,
                &mut b,
            );
            assert_eq!(0, b.size());
            assert!(r == Ok((public_payload, masked_payload)));
        }
    }
}
