import { useState, useEffect } from 'react';

const useCountdown = (targetDate: string, serverTimeLeftInSeconds?: number) => {
  const calculateTimeLeftFromTarget = () => {
    if (!targetDate) return 0;
    const countDownDate = new Date(targetDate).getTime();
    return countDownDate - new Date().getTime();
  };

  const [countDown, setCountDown] = useState(calculateTimeLeftFromTarget());

  useEffect(() => {
    // If a server time is provided, sync the countdown with it.
    if (serverTimeLeftInSeconds !== undefined && serverTimeLeftInSeconds >= 0) {
      setCountDown(serverTimeLeftInSeconds * 1000);
    } else {
      // Otherwise, recalculate from the target date when the component mounts or targetDate changes.
      setCountDown(calculateTimeLeftFromTarget());
    }
  }, [serverTimeLeftInSeconds, targetDate]);

  useEffect(() => {
    // The interval ticks the countdown down every second locally.
    const interval = setInterval(() => {
      setCountDown((prevCountDown) => (prevCountDown > 0 ? prevCountDown - 1000 : 0));
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return getReturnValues(countDown);
};

const getReturnValues = (countDown: number) => {
  if (countDown <= 0) {
    return { days: 0, hours: 0, minutes: 0, seconds: 0, isFinished: true };
  }
  const days = Math.floor(countDown / (1000 * 60 * 60 * 24));
  const hours = Math.floor((countDown % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
  const minutes = Math.floor((countDown % (1000 * 60 * 60)) / (1000 * 60));
  const seconds = Math.floor((countDown % (1000 * 60)) / 1000);

  return { days, hours, minutes, seconds, isFinished: false };
};

export { useCountdown };
