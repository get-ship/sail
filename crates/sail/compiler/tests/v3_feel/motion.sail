<scope>
  style toastCard = {
    size({ min-w: 80 }).pad({ x: 4, y: 3 }).bg(card).border(subtle).radius(lg).elevation(lg),
    animate: {
      enter: play(slide-up, fast),
      exit: play(fade, fast),
      transition: play(colors, fast),
    },
  }
  style liftCard = {
    bg(card).border(subtle).radius(lg).elevation(sm),
    animate: { transition: play({ props: all, duration: fast, ease: out }) },
    when: {
      hover: elevation(xl).translateY(-1),
      press: scale(0_98),
    },
  }
</scope>

<toast-region placement="bottom-right">
  <for {toasts()} as="t" key={t.id}>
    <toast tone={t.tone} style(toastCard)>{t.message}</toast>
  </for>
</toast-region>

<card style(liftCard)>
  <img src={item.thumb} alt={item.title}
       style(size(full).radius({ t: lg })) />
  <column gap="1" style(pad({ x: 4, y: 3 }))>
    <h3>{item.title}</h3>
    <p style(type(body-sm).fg(muted-foreground))>{item.blurb}</p>
  </column>
</card>
