package io.casperlabs.node.api

import cats.implicits._
import cats.Monad
import com.google.protobuf.ByteString
import io.casperlabs.casper.consensus.state
import io.casperlabs.catscontrib.MonadThrowable
import io.casperlabs.crypto.codec.Base16
import io.casperlabs.crypto.hash.Blake2b256

import scala.util.Try

object Utils {
  def check[F[_]: MonadThrowable, A](
      s: A,
      desc: String,
      b: A => Boolean
  ): F[A] =
    MonadThrowable[F].raiseError[A](new IllegalArgumentException(s"$desc")).whenA(!b(s)) >> s
      .pure[F]

  def validateBlockHashPrefix[F[_]: MonadThrowable](
      p: String,
      adaptError: PartialFunction[Throwable, Throwable] = PartialFunction.empty
  ): F[String] =
    Utils
      .check[F, String](
        p,
        "BlockHash prefix must be >= 4 and <= 64 base16 characters (2 to 32 bytes) long",
        _.matches("[a-f0-9]{4,64}")
      )
      .adaptErr(adaptError)

  def validateDeployHash[F[_]: MonadThrowable](
      p: String,
      adaptError: PartialFunction[Throwable, Throwable] = PartialFunction.empty
  ): F[String] =
    Utils
      .check[F, String](
        p,
        "DeployHash must be 64 base16 characters (32 bytes) long",
        _.matches("[a-f0-9]{64}")
      )
      .adaptError(adaptError)

  def validateAccountPublicKey[F[_]: MonadThrowable](
      p: String,
      adaptError: PartialFunction[Throwable, Throwable] = PartialFunction.empty
  ): F[String] =
    Utils
      .check[F, String](
        p,
        "AccountPublicKey must be 64 base16 characters (32 bytes) long",
        _.matches("[a-f0-9]{64}")
      )
      .adaptError(adaptError)

  def toKey[F[_]: Monad](keyType: String, keyValue: String)(
      implicit appErr: MonadThrowable[F]
  ): F[state.Key] = {
    val keyBytes = ByteString.copyFrom(Base16.decode(keyValue))
    keyType.toLowerCase match {
      case "hash" =>
        keyBytes.size match {
          case 32 => state.Key(state.Key.Value.Hash(state.Key.Hash(keyBytes))).pure[F]
          case n =>
            appErr.raiseError(
              new IllegalArgumentException(
                s"Key of type hash must have exactly 32 bytes, $n =/= 32 provided."
              )
            )
        }
      case "uref" =>
        keyBytes.size match {
          case 32 => state.Key(state.Key.Value.Uref(state.Key.URef(keyBytes))).pure[F]
          case n =>
            appErr.raiseError(
              new IllegalArgumentException(
                s"Key of type uref must have exactly 32 bytes, $n =/= 32 provided."
              )
            )
        }
      case "address" =>
        keyBytes.size match {
          case 32 => state.Key(state.Key.Value.Address(state.Key.Address(keyBytes))).pure[F]
          case n =>
            appErr.raiseError(
              new IllegalArgumentException(
                s"Key of type address must have exactly 32 bytes, $n =/= 32 provided."
              )
            )
        }
      case "local" =>
        for {
          (seed, bytes) <- appErr
                            .fromTry {
                              Try(keyValue.split(':').map(Base16.decode(_)))
                                .filter(_.length == 2)
                                .map(arr => (arr(0), arr(1)))
                            }
                            .handleErrorWith(
                              _ =>
                                appErr.raiseError(
                                  new IllegalArgumentException(
                                    "Expected local key encoded as {seed}:{rest}. Where both seed and rest parts are hex encoded."
                                  )
                                )
                            )
          _ <- appErr
                .raiseError(
                  new IllegalArgumentException("Seed of Local key has to be exactly 32 bytes long.")
                )
                .whenA(seed.length != 32)
        } yield {
          // This is what EE does when creating local key address.
          val hash = Blake2b256.hash(seed ++ bytes)
          state.Key(state.Key.Value.Local(state.Key.Local(ByteString.copyFrom(hash))))
        }
      case _ =>
        appErr.raiseError(
          new IllegalArgumentException(
            s"Key variant $keyType not valid. Must be one of hash, uref, address."
          )
        )
    }
  }
}
