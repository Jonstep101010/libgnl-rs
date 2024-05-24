/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   get_next_line.c                                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: jschwabe <jschwabe@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2023/04/18 14:25:50 by jschwabe          #+#    #+#             */
/*   Updated: 2024/05/24 19:22:45 by jschwabe         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "get_next_line.h"
#include <limits.h>

static char	*check_free(char *line)
{
	char	*tmp;
	int		i;

	if (!line)
		return (NULL);
	i = 0;
	while (line[i] != '\0')
		i++;
	tmp = ft_calloc(sizeof(char), i + 1);
	if (!tmp)
		return (free(line), NULL);
	ft_memcpy(tmp, line, i);
	free(line);
	return (tmp);
}

static char	*read_line(char *buf, int fd, int *counter, char **line);
static void	clean_buffer(char *buf);

char	*get_next_line(int fd)
{
	char			*line;
	static char		buf[BUFFER_SIZE + 1];
	int				counter;

	if (fd < 0 || BUFFER_SIZE < 1)
		return (NULL);
	line = NULL;
	counter = -1;
	while (++counter < BUFFER_SIZE && buf[counter])
	{
		if (buf[counter] == '\n')
		{
			line = ft_calloc(sizeof(char), BUFFER_SIZE + 1);
			if (!line)
				return (NULL);
			ft_memcpy(line, buf, counter + 1);
			clean_buffer(buf);
			return (check_free(line));
		}
	}
	if (buf[counter] != '\n')
		read_line(buf, fd, &counter, &line);
	return (check_free(line));
}

static void	clean_buffer(char *buf)
{
	int	nl_index;

	nl_index = 0;
	while (nl_index < INT_MAX
		&& buf[nl_index] != '\n' && buf[nl_index] != '\0')
		nl_index++;
	if (buf[nl_index] != '\n')
		buf[nl_index] = 0;
	else
		nl_index++;
	ft_memcpy(buf, (buf + nl_index), (BUFFER_SIZE - nl_index) + 1);
}

static void	read_success(char *tmp, char **line, int *counter)
{
	int	i;

	*counter -= BUFFER_SIZE;
	i = 0;
	while (i < BUFFER_SIZE && tmp[i] != '\n' && tmp[i] != '\0')
		i++;
	ft_memcpy((*line + *counter), tmp, i);
	if (tmp[i] == '\n')
		(*line)[*counter + i] = '\n';
}

static char	*read_line(char *buf, int fd, int *counter, char **line)
{
	char		tmp[BUFFER_SIZE + 1];
	const int	rd = read(fd, ft_memset(tmp, 0, BUFFER_SIZE), BUFFER_SIZE);
	int			i;

	if (rd == -1)
		return (ft_memset(buf, 0, BUFFER_SIZE));
	if (rd > 0)
		*counter += BUFFER_SIZE;
	i = 0;
	while (i < BUFFER_SIZE && tmp[i] != '\n' && tmp[i] != '\0')
		i++;
	if (tmp[i] == '\n' || (rd == 0 && *counter != 0))
	{
		*line = ft_calloc(sizeof(char), *counter + 1);
		if (!*line)
			return (NULL);
		ft_strlcpy(*line, buf, *counter + 1);
		ft_memcpy(buf, tmp, BUFFER_SIZE);
		clean_buffer(buf);
	}
	if (tmp[i] != '\n' && rd != 0 && !read_line(buf, fd, counter, line))
		return (NULL);
	else if (rd > 0)
		read_success(tmp, line, counter);
	return (*line);
}
